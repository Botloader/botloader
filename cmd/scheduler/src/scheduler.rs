use std::{
    collections::{HashMap, HashSet},
    pin::Pin,
    sync::Arc,
    task::Poll,
};

use crate::{
    command_manager,
    guild_handler::{GuildCommand, GuildHandle, GuildHandler, GuildHandlerEvent},
};
use dbrokerapi::broker_scheduler_rpc::{GuildEvent, HelloData};
use std::future::Future;
use tokio::sync::mpsc;
use tracing::info;
use twilight_model::{gateway::event::DispatchEvent, id::GuildId};

pub enum SchedulerCommand {
    BrokerConnected,
    BrokerDisconnected,
    BrokerHello(HelloData),
    DiscordEvent(GuildEvent),
    Shutdown,
    ReloadGuildScripts(GuildId),
}

pub struct Scheduler {
    guilds: HashMap<GuildId, GuildHandle>,
    cmd_rx: mpsc::UnboundedReceiver<SchedulerCommand>,
    queued_events: Vec<GuildEvent>,
    pending_starts: Vec<GuildId>,
    stores: Arc<dyn Store>,
    logger: guild_logger::GuildLogger,
    cmd_manager_handle: command_manager::Handle,
    worker_pool: crate::vmworkerpool::VmWorkerPool,
    discord_client: Arc<twilight_http::Client>,

    // guilds that had their vm's forcibly shut down
    shutdown_guilds: HashSet<GuildId>,
}

impl Scheduler {
    pub fn new(
        scheduler_rx: mpsc::UnboundedReceiver<SchedulerCommand>,
        stores: Arc<dyn Store>,
        logger: guild_logger::GuildLogger,
        cmd_manager_handle: command_manager::Handle,
        worker_pool: crate::vmworkerpool::VmWorkerPool,
        discord_client: Arc<twilight_http::Client>,
    ) -> Self {
        Self {
            stores,
            logger,
            cmd_manager_handle,
            worker_pool,
            discord_client,

            guilds: HashMap::new(),
            cmd_rx: scheduler_rx,
            queued_events: Vec::new(),
            pending_starts: Vec::new(),
            shutdown_guilds: HashSet::new(),
        }
    }

    pub async fn run(mut self) {
        loop {
            match self.next_action().await {
                SchedulerAction::Cmd(Some(SchedulerCommand::Shutdown)) => {
                    break;
                }
                SchedulerAction::Cmd(Some(cmd)) => self.handle_scheduler_command(cmd).await,
                // rpc server shut down, shut down scheduler aswell then
                SchedulerAction::Cmd(None) => break,

                SchedulerAction::GuildHandler(guild_id, None) => {
                    // worker finished, remove it
                    self.guilds.remove(&guild_id);
                    if !self.shutdown_guilds.contains(&guild_id) {
                        self.check_queue_start_worker(guild_id).await;
                    }
                }
                SchedulerAction::GuildHandler(g, Some(evt)) => {
                    self.handle_guild_handler_event(g, evt)
                }
            }
        }

        self.shutdown_all();
        self.wait_all_shutdown().await;
    }

    async fn wait_all_shutdown(&mut self) {
        info!("shutdown pending guilds: {}", self.guilds.len());
        if self.guilds.is_empty() {
            return;
        }

        loop {
            if let SchedulerAction::GuildHandler(guild_id, None) = self.next_action().await {
                // worker finished, remove it
                self.guilds.remove(&guild_id);
                self.check_queue_start_worker(guild_id).await;

                info!("shutdown pending guilds: {}", self.guilds.len());

                if self.guilds.is_empty() {
                    return;
                }
            }
        }
    }

    async fn next_action(&mut self) -> SchedulerAction {
        SchedulerNextActionFuture { scheduler: self }.await
    }

    async fn check_queue_start_worker(&mut self, guild_id: GuildId) {
        if let Some(index) = self
            .pending_starts
            .iter()
            .enumerate()
            .find_map(|(i, v)| (*v == guild_id).then(|| i))
        {
            self.pending_starts.remove(index);
            self.get_or_start_guild(guild_id);
        }

        // god i wish drain_filter was stable
        let mut indexes = self
            .queued_events
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                if v.guild_id == guild_id {
                    Some(i)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // reverse to avoid inavlidating indexes as we remove their items
        indexes.reverse();

        let mut evts = indexes
            .into_iter()
            .map(|i| self.queued_events.remove(i))
            .collect::<Vec<_>>();

        // reverse back in the proper order
        evts.reverse();

        for evt in evts {
            self.send_or_queue_broker_evt(evt);
            // self.handle_broker_evt(evt).await;
        }
    }

    fn handle_guild_handler_event(&mut self, guild_id: GuildId, event: GuildHandlerEvent) {
        match event {
            GuildHandlerEvent::ForciblyShutdown => {
                info!("guild {} forcibly shut down, blacklisting it", guild_id);
                self.shutdown_guilds.insert(guild_id);

                // remove all queued starts and events
                let new_pending_starts = self
                    .pending_starts
                    .iter()
                    .filter_map(|v| (*v != guild_id).then(|| *v))
                    .collect::<Vec<_>>();
                self.pending_starts = new_pending_starts;

                let old_events = std::mem::take(&mut self.queued_events);
                self.queued_events = old_events
                    .into_iter()
                    .filter(|v| v.guild_id != guild_id)
                    .collect();
            }
        }
    }

    async fn handle_scheduler_command(&mut self, cmd: SchedulerCommand) {
        match cmd {
            // we shut down all previously running workers when a new broker connects
            // this is because we could be out of sync otherwise and running guilds we shouldn't run
            SchedulerCommand::BrokerHello(d) => {
                info!("new broker connected");
                self.shutdown_all();

                self.pending_starts = Vec::new();

                // start all the workers we can
                for g in d.connected_guilds {
                    if self.shutdown_guilds.contains(&g) {
                        continue;
                    }

                    let worker = self.get_or_start_guild(g);
                    if worker.tx.is_none() {
                        // this worker shutting down, schedule it for restart
                        self.pending_starts.push(g)
                    }
                }
            }

            SchedulerCommand::BrokerDisconnected => {
                self.shutdown_all();
            }
            SchedulerCommand::BrokerConnected => {}
            SchedulerCommand::DiscordEvent(evt) => {
                if self.shutdown_guilds.contains(&evt.guild_id) {
                    return;
                }

                if let DispatchEvent::GuildDelete(_) = *evt.event {
                    if let Some(worker) = self.guilds.get_mut(&evt.guild_id) {
                        // this will signal the worker to shut down
                        if let Some(tx) = worker.tx.take() {
                            let _ = tx.send(GuildCommand::Shutdown);
                        }
                    }
                } else {
                    self.send_or_queue_broker_evt(evt)
                }
            }
            SchedulerCommand::Shutdown => {
                panic!("should be handled by caller")
            }
            SchedulerCommand::ReloadGuildScripts(guild_id) => {
                // reset the blacklisted state
                if self.shutdown_guilds.remove(&guild_id) {
                    self.get_or_start_guild(guild_id);
                }

                if let Some(g) = self.guilds.get(&guild_id) {
                    if let Some(tx) = &g.tx {
                        let _ = tx.send(GuildCommand::ReloadScripts);
                    }
                }
            }
        }
    }

    fn send_or_queue_broker_evt(&mut self, evt: GuildEvent) {
        let worker = self.get_or_start_guild(evt.guild_id);

        if let Some(tx) = &worker.tx {
            if let Err(e) = tx.send(GuildCommand::BrokerEvent(evt)) {
                // dropped, push it to the queue
                if let GuildCommand::BrokerEvent(evt) = e.0 {
                    self.queued_events.push(evt);
                }
            }
        } else {
            // in shutting down state
            self.queued_events.push(evt);
        }
    }

    // if this function body looks weird to you, then you're correct, it is weird.
    // reason for that being the borrow checker, try to make it look prettier, i dare you.
    fn get_or_start_guild(&mut self, guild_id: GuildId) -> &GuildHandle {
        if let std::collections::hash_map::Entry::Vacant(e) = self.guilds.entry(guild_id) {
            let handle = GuildHandler::new_run(
                self.stores.clone(),
                guild_id,
                self.logger.clone(),
                self.worker_pool.clone(),
                self.cmd_manager_handle.clone(),
                self.discord_client.clone(),
            );
            e.insert(handle);
            return self.guilds.get(&guild_id).unwrap();
        }

        if let Some(worker) = self.guilds.get(&guild_id) {
            worker
        } else {
            unreachable!();
        }
    }

    fn shutdown_all(&mut self) {
        for worker in self.guilds.values_mut() {
            if let Some(tx) = worker.tx.take() {
                let _ = tx.send(GuildCommand::Shutdown);
            }
        }
    }
}

enum SchedulerAction {
    Cmd(Option<SchedulerCommand>),
    GuildHandler(GuildId, Option<GuildHandlerEvent>),
}

struct SchedulerNextActionFuture<'a> {
    scheduler: &'a mut Scheduler,
}

impl<'a> Future for SchedulerNextActionFuture<'a> {
    type Output = SchedulerAction;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let fut = self.get_mut();

        if !fut.scheduler.guilds.is_empty() {
            for (guild_id, handle) in &mut fut.scheduler.guilds {
                if let Poll::Ready(r) = handle.rx.poll_recv(cx) {
                    return Poll::Ready(SchedulerAction::GuildHandler(*guild_id, r));
                }
            }
        }

        if let Poll::Ready(cmd) = fut.scheduler.cmd_rx.poll_recv(cx) {
            return Poll::Ready(SchedulerAction::Cmd(cmd));
        }

        Poll::Pending
    }
}

pub trait Store: stores::config::ConfigStore + stores::timers::TimerStore {}

impl<T: stores::config::ConfigStore + stores::timers::TimerStore> Store for T {}

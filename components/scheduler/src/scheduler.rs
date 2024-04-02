use std::{
    collections::HashMap,
    pin::Pin,
    sync::Arc,
    task::Poll,
    time::{Duration, Instant},
};

use crate::{
    command_manager,
    guild_handler::{GuildCommand, GuildHandle, GuildHandler, GuildStatus},
    vm_session::VmSessionEvent,
    vmworkerpool::WorkerStatus,
    SchedulerConfig,
};
use dbrokerapi::broker_scheduler_rpc::{DiscordEvent, DiscordEventData, HelloData};
use guild_logger::LogEntry;
use std::future::Future;
use stores::Db;
use tokio::sync::{mpsc, oneshot};
use tracing::info;
use twilight_model::id::{marker::GuildMarker, Id};

pub enum SchedulerCommand {
    BrokerConnected,
    BrokerDisconnected,
    BrokerHello(HelloData),
    DiscordEvent(DiscordEvent),
    Shutdown,
    ReloadGuildScripts(Id<GuildMarker>),
    PurgeGuildCache(Id<GuildMarker>),
    WorkerStatus(oneshot::Sender<Vec<WorkerStatus>>),
    GuildStatus(Id<GuildMarker>, oneshot::Sender<Option<GuildStatus>>),
}

pub struct Scheduler {
    guilds: HashMap<Id<GuildMarker>, GuildHandle>,
    cmd_rx: mpsc::UnboundedReceiver<SchedulerCommand>,
    queued_events: Vec<DiscordEvent>,
    pending_starts: Vec<Id<GuildMarker>>,
    stores: Db,
    logger: guild_logger::LogSender,
    cmd_manager_handle: command_manager::Handle,
    worker_pool: crate::vmworkerpool::VmWorkerPool,
    config: Arc<SchedulerConfig>,

    suspended_guilds: HashMap<Id<GuildMarker>, GuildSuspension>,
}

impl Scheduler {
    pub fn new(
        config: Arc<SchedulerConfig>,
        scheduler_rx: mpsc::UnboundedReceiver<SchedulerCommand>,
        stores: Db,
        logger: guild_logger::LogSender,
        cmd_manager_handle: command_manager::Handle,
        worker_pool: crate::vmworkerpool::VmWorkerPool,
    ) -> Self {
        Self {
            stores,
            logger,
            cmd_manager_handle,
            worker_pool,
            config,

            guilds: HashMap::new(),
            cmd_rx: scheduler_rx,
            queued_events: Vec::new(),
            pending_starts: Vec::new(),
            suspended_guilds: HashMap::new(),
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

                    if self.try_unsuspend_guild(guild_id) {
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

    async fn check_queue_start_worker(&mut self, guild_id: Id<GuildMarker>) {
        if let Some(index) = self.pending_starts.iter().position(|v| *v == guild_id) {
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

        // reverse to avoid invalidating indexes as we remove their items
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

    fn handle_guild_handler_event(&mut self, guild_id: Id<GuildMarker>, event: VmSessionEvent) {
        match event {
            VmSessionEvent::ShutdownExcessCpu => {
                info!(
                    "guild {} forcibly shut down for excess cpu usage, blacklisting it",
                    guild_id
                );
                self.mark_guild_as_suspended(guild_id, SuspensionReason::ExcessCpu);
            }
            VmSessionEvent::ShutdownTooManyInvalidRequests => {
                info!(
                    "guild {} forcibly shut down from too many invalid requests, blacklisting it",
                    guild_id
                );
                self.mark_guild_as_suspended(
                    guild_id,
                    SuspensionReason::ExcessInvalidDiscordRequests,
                );
            }
        }
    }

    fn mark_guild_as_suspended(&mut self, guild_id: Id<GuildMarker>, reason: SuspensionReason) {
        info!("guild {guild_id} marked as suspended, reason: {reason:?}");
        self.suspended_guilds
            .insert(guild_id, GuildSuspension::new(guild_id, reason));

        // remove all queued starts and events for this guild
        self.pending_starts.retain(|v| *v != guild_id);
        self.queued_events.retain(|v| v.guild_id != guild_id);
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
                    if !self.try_unsuspend_guild(g) {
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
                if !self.try_unsuspend_guild(evt.guild_id) {
                    return;
                }

                if let DiscordEventData::GuildDelete(_) = evt.event {
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
                if self.try_unsuspend_guild(guild_id) {
                    self.get_or_start_guild(guild_id);
                } else {
                    self.logger.log(LogEntry::error(
                        guild_id,
                        "can't unsuspend your guild yet, please wait 10 minutes".to_owned(),
                    ));
                    return;
                }

                if let Some(g) = self.guilds.get(&guild_id) {
                    if let Some(tx) = &g.tx {
                        let _ = tx.send(GuildCommand::ReloadScripts);
                    }
                }
            }
            SchedulerCommand::PurgeGuildCache(guild_id) => {
                if let Some(g) = self.guilds.get(&guild_id) {
                    if let Some(tx) = &g.tx {
                        let _ = tx.send(GuildCommand::PurgeCache);
                    }
                }
            }
            SchedulerCommand::WorkerStatus(req) => {
                let statuses = self.worker_pool.worker_statuses();
                let _ = req.send(statuses);
            }
            SchedulerCommand::GuildStatus(guild_id, resp) => {
                if let Some(g) = self.guilds.get(&guild_id) {
                    if let Some(tx) = &g.tx {
                        let _ = tx.send(GuildCommand::Status(resp));
                        return;
                    }
                }
                let _ = resp.send(None);
            }
        }
    }

    fn send_or_queue_broker_evt(&mut self, evt: DiscordEvent) {
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

    fn get_or_start_guild(&mut self, guild_id: Id<GuildMarker>) -> &GuildHandle {
        self.guilds.entry(guild_id).or_insert_with(|| {
            GuildHandler::new_run(
                self.config.clone(),
                self.stores.clone(),
                guild_id,
                self.logger.clone(),
                self.worker_pool.clone(),
                self.cmd_manager_handle.clone(),
            )
        })
    }

    fn shutdown_all(&mut self) {
        for worker in self.guilds.values_mut() {
            if let Some(tx) = worker.tx.take() {
                let _ = tx.send(GuildCommand::Shutdown);
            }
        }
    }

    fn try_unsuspend_guild(&mut self, guild_id: Id<GuildMarker>) -> bool {
        if let Some(susp) = self.suspended_guilds.get(&guild_id) {
            if susp.can_remove() {
                self.suspended_guilds.remove(&guild_id);
            } else {
                return false;
            }
        }

        true
    }
}

enum SchedulerAction {
    Cmd(Option<SchedulerCommand>),
    GuildHandler(Id<GuildMarker>, Option<VmSessionEvent>),
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

#[derive(Debug)]
enum SuspensionReason {
    ExcessCpu,
    ExcessInvalidDiscordRequests,
}

impl SuspensionReason {
    fn duration(&self) -> Duration {
        match self {
            Self::ExcessCpu => Duration::from_secs(15),
            Self::ExcessInvalidDiscordRequests => Duration::from_secs(60 * 10),
        }
    }
}

struct GuildSuspension {
    _guild_id: Id<GuildMarker>,
    reason: SuspensionReason,
    suspended_at: Instant,
}

impl GuildSuspension {
    fn new(guild_id: Id<GuildMarker>, reason: SuspensionReason) -> Self {
        Self {
            _guild_id: guild_id,
            reason,
            suspended_at: Instant::now(),
        }
    }

    fn can_remove(&self) -> bool {
        let dur = self.reason.duration();

        dur < self.suspended_at.elapsed()
    }
}

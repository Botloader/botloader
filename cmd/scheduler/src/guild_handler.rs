use std::{collections::HashMap, sync::Arc, task::Poll, time::Duration};

use crate::{
    command_manager, interval_timer_manager, scheduled_task_manager, scheduler::Store,
    vmworkerpool::WorkerHandle,
};
use chrono::{DateTime, Utc};
use common::DiscordConfig;
use dbrokerapi::broker_scheduler_rpc::GuildEvent;
use guild_logger::{GuildLogger, LogEntry};
use runtime_models::internal::script::ScriptMeta;
use scheduler_worker_rpc::{
    MetricEvent, RunStateChangeReq, SchedulerMessage, UpdateRunStateRequest, VmDispatchEvent,
    WorkerMessage,
};
use std::future::Future;
use stores::{
    config::{IntervalTimerContrib, Script, ScriptContributes},
    timers::{IntervalTimer, ScheduledTask},
};
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, instrument};
use twilight_model::{
    gateway::event::DispatchEvent,
    id::{marker::GuildMarker, Id},
};

pub enum GuildCommand {
    BrokerEvent(GuildEvent),
    // Dispatch(oneshot::Sender<()>, String, serde_json::Value),
    ReloadScripts,
    Shutdown,
}

pub struct GuildHandler {
    guild_id: Id<GuildMarker>,

    discord_config: Arc<DiscordConfig>,
    stores: Arc<dyn Store>,
    logger: GuildLogger,
    worker_pool: crate::vmworkerpool::VmWorkerPool,
    scheduler_tx: mpsc::UnboundedSender<GuildHandlerEvent>,
    guild_rx: mpsc::UnboundedReceiver<GuildCommand>,
    interval_timers_man: crate::interval_timer_manager::Manager,
    cmd_manager_handle: command_manager::Handle,
    scheduled_tasks_man: scheduled_task_manager::Manager,

    pending_acks: HashMap<u64, PendingAck>,
    current_worker: Option<WorkerHandle>,
    force_load_scripts_next: bool,
    scripts: Vec<Script>,
    id_gen: u64,
}

impl GuildHandler {
    pub fn new_run(
        stores: Arc<dyn Store>,
        guild_id: Id<GuildMarker>,
        logger: GuildLogger,
        worker_pool: crate::vmworkerpool::VmWorkerPool,
        cmd_manager_handle: crate::command_manager::Handle,
        discord_config: Arc<DiscordConfig>,
    ) -> GuildHandle {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let (evt_tx, evt_rx) = mpsc::unbounded_channel();
        let handle = GuildHandle {
            guild_id,

            rx: evt_rx,
            tx: Some(cmd_tx),
        };

        let interval_timer_man =
            crate::interval_timer_manager::Manager::new(guild_id, stores.clone());

        let tasks_man = scheduled_task_manager::Manager::new(guild_id, stores.clone());

        let worker = GuildHandler {
            stores,
            guild_id,
            logger,
            worker_pool,
            discord_config,

            guild_rx: cmd_rx,
            scheduler_tx: evt_tx,
            id_gen: 1,
            pending_acks: HashMap::new(),
            current_worker: None,
            scripts: Vec::new(),
            force_load_scripts_next: false,

            interval_timers_man: interval_timer_man,
            cmd_manager_handle,
            scheduled_tasks_man: tasks_man,
        };

        tokio::spawn(worker.run());

        handle
    }

    #[instrument(skip(self), fields(guild_id = self.guild_id.get()))]
    async fn run(mut self) {
        self.try_retry_load_scripts().await;
        self.load_contribs().await;

        while let Some(next) = self.next_event().await {
            match next {
                NextGuildAction::GuildCommand(GuildCommand::Shutdown) => {
                    info!("got shutdown signal");
                    break;
                }
                NextGuildAction::GuildCommand(cmd) => {
                    self.handle_guild_command(cmd).await;
                }
                NextGuildAction::WorkerMessage(Some(WorkerMessage::Shutdown(reason))) => {
                    self.logger.log(LogEntry::critical(
                        self.guild_id,
                        format!("vm was forcibly shut down, reason: {:?}", reason),
                    ));
                    let _ = self.scheduler_tx.send(GuildHandlerEvent::ForciblyShutdown);
                    break;
                }
                NextGuildAction::WorkerMessage(Some(msg)) => {
                    self.handle_worker_msg(msg).await;
                }
                NextGuildAction::WorkerMessage(None) => {
                    self.broken_worker().await;
                }
                NextGuildAction::TimerCheck => continue,
                NextGuildAction::DispatchScheduledTasks(tasks) => {
                    for task in tasks {
                        self.dispatch_scheduled_task(task).await;
                    }
                }
                NextGuildAction::DispatchIntervalTimers(timers) => {
                    for timer in timers {
                        self.dispatch_interval_timer(timer).await;
                    }
                }
            }
        }

        info!("shutting down guild handler");

        // self.shutdown_vm_wait().await;
        if let Some(worker) = self.current_worker.take() {
            self.worker_pool.return_worker(worker, false);
        }
    }

    async fn load_contribs(&mut self) {
        info!("loading contribs");

        let evt_id = self.gen_id();
        if let Some(worker) = &self.current_worker {
            if worker
                .tx
                .send(SchedulerMessage::UpdateRunState(
                    evt_id,
                    UpdateRunStateRequest {
                        guild_id: self.guild_id,
                        guild_scripts: RunStateChangeReq::Start(self.scripts.clone()),
                        packs: RunStateChangeReq::Stop,
                    },
                ))
                .is_err()
            {
                self.broken_worker().await;
            }
        } else {
            self.reset_contribs();

            if self.scripts.is_empty() {
                return;
            }

            self.force_load_scripts_next = true;
            self.ensure_claim_worker().await;
        }
    }

    async fn try_retry_load_scripts(&mut self) {
        loop {
            match self.stores.list_scripts(self.guild_id).await {
                Ok(scripts) => {
                    self.scripts = scripts.into_iter().filter(|v| v.enabled).collect();
                    return;
                }
                Err(err) => {
                    error!(%err, "failed loading guild scripts, retrying in 10 secs");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            }
        }
    }

    async fn next_event(&mut self) -> Option<NextGuildAction> {
        let scheduled_task_sleep_check = match self.scheduled_tasks_man.next_action().await {
            scheduled_task_manager::NextAction::None => {
                tokio::time::sleep(Duration::from_secs(60 * 60))
            }
            scheduled_task_manager::NextAction::Wait(until) => {
                let sleep_dur = (until - chrono::Utc::now())
                    .to_std()
                    .unwrap_or_else(|_| Duration::from_millis(1));
                tokio::time::sleep(sleep_dur)
            }
            scheduled_task_manager::NextAction::Run(v) => {
                return Some(NextGuildAction::DispatchScheduledTasks(v));
            }
        };

        let interval_timers_sleep_check = match self.interval_timers_man.next_action().await {
            interval_timer_manager::NextAction::None => {
                tokio::time::sleep(Duration::from_secs(60 * 60))
            }
            interval_timer_manager::NextAction::Wait(until) => {
                let sleep_dur = (until - chrono::Utc::now())
                    .to_std()
                    .unwrap_or_else(|_| Duration::from_millis(1));
                tokio::time::sleep(sleep_dur)
            }
            interval_timer_manager::NextAction::Run(v) => {
                return Some(NextGuildAction::DispatchIntervalTimers(v));
            }
        };

        let next_cmd_fut = NextGuildHandlerActionFuture {
            guild_handler: self,
        };

        tokio::select! {
            _ = scheduled_task_sleep_check  => Some(NextGuildAction::TimerCheck),
            _ = interval_timers_sleep_check  => Some(NextGuildAction::TimerCheck),
            act = next_cmd_fut => act,
        }
    }

    async fn handle_worker_msg(&mut self, msg: WorkerMessage) {
        match msg {
            WorkerMessage::Ack(id) => {
                if let Some(item) = self.pending_acks.remove(&id) {
                    match item {
                        PendingAck::Dispatch(Some(resp)) => {
                            let _ = resp.send(());
                        }
                        PendingAck::Dispatch(_) => {}
                        PendingAck::ScheduledTask(t_id) => {
                            self.scheduled_tasks_man.ack_triggered_task(t_id).await;
                        }
                        PendingAck::IntervalTimer(name) => {
                            self.interval_timers_man.timer_ack(name).await;
                        }
                    }
                }
            }
            WorkerMessage::ScriptStarted(start) => {
                self.script_loaded(start).await;
            }
            WorkerMessage::ScriptsInit => todo!(),
            WorkerMessage::NonePending => {
                if self.pending_acks.is_empty() {
                    if let Some(current) = self.current_worker.take() {
                        // return worker
                        self.worker_pool.return_worker(current, false);
                    }
                }
            }
            WorkerMessage::TaskScheduled => {
                self.scheduled_tasks_man.clear_next();
            }
            WorkerMessage::GuildLog(entry) => {
                self.logger.log(entry);
            }
            WorkerMessage::Hello(_) => {
                // handled when connection is established, not applicable here
                unreachable!();
            }
            WorkerMessage::Shutdown(_) => {
                // handled in parent
                unreachable!();
            }
            WorkerMessage::Metric(name, m, labels) => self.handle_metric(name, m, labels),
        }
    }

    fn handle_metric(&mut self, name: String, m: MetricEvent, labels: HashMap<String, String>) {
        let recorder = if let Some(rec) = metrics::try_recorder() {
            rec
        } else {
            return;
        };

        let mut labels = labels
            .into_iter()
            .map(|(k, v)| metrics::Label::new(k, v))
            .collect::<Vec<_>>();

        labels.push(metrics::Label::new("guild_id", self.guild_id.to_string()));

        let key = metrics::Key::from_parts(name, labels);

        match m {
            MetricEvent::Gauge(action) => {
                let handle = recorder.register_gauge(&key);
                match action {
                    scheduler_worker_rpc::GaugeEvent::Set(v) => handle.set(v),
                    scheduler_worker_rpc::GaugeEvent::Incr(v) => handle.increment(v),
                }
            }
            MetricEvent::Counter(action) => {
                let handle = recorder.register_counter(&key);

                match action {
                    scheduler_worker_rpc::CounterEvent::Incr(v) => handle.increment(v),
                    scheduler_worker_rpc::CounterEvent::Absolute(v) => handle.absolute(v),
                }
            }
        }
    }

    async fn handle_guild_command(&mut self, cmd: GuildCommand) {
        match cmd {
            GuildCommand::BrokerEvent(evt) => {
                self.handle_broker_event(evt).await;
            }
            // GuildCommand::Dispatch(resp, t, v) => {
            //     self.dispatch_worker_evt(t, v, PendingAck::Dispatch(Some(resp)))
            //         .await;
            // }
            GuildCommand::ReloadScripts => {
                self.try_retry_load_scripts().await;
                self.load_contribs().await;
            }
            GuildCommand::Shutdown => {
                panic!("shutdown should be handled by caller")
            }
        }
    }

    async fn dispatch_scheduled_task(&mut self, task: ScheduledTask) {
        info!("dispatching scheduled task");
        let task_id = task.id;
        let evt = runtime_models::internal::tasks::ScheduledTask::from(task);
        let serialized = serde_json::to_value(&evt).unwrap();
        self.dispatch_worker_evt(
            "BOTLOADER_SCHEDULED_TASK_FIRED".to_string(),
            serialized,
            PendingAck::ScheduledTask(task_id),
        )
        .await;
    }

    async fn dispatch_interval_timer(&mut self, timer: IntervalTimer) {
        info!("dispatching interval timer");
        let evt = runtime_models::internal::timers::IntervalTimerEvent {
            name: timer.name.clone(),
        };

        let serialized = serde_json::to_value(&evt).unwrap();
        self.dispatch_worker_evt(
            "BOTLOADER_INTERVAL_TIMER_FIRED".to_string(),
            serialized,
            PendingAck::IntervalTimer(timer.name),
        )
        .await;
    }

    async fn handle_broker_event(&mut self, evt: GuildEvent) {
        match &*evt.event {
            DispatchEvent::GuildCreate(_) => {}
            DispatchEvent::GuildDelete(_) => {
                unreachable!("this event should not be forwarded to the guild worker");
            }
            // DispatchEvent::InteractionCreate(interaction) => {
            //     // potentially ack the command
            //     // TODO: should this perhaps be done on the js side?
            //     if let twilight_model::application::interaction::Interaction::ApplicationCommand(
            //         interaction,
            //     ) = &interaction.0
            //     {
            //         let interaction_client = self.discord_config.interaction_client();
            //         if let Err(err) = interaction_client
            //             .interaction_callback(
            //                 interaction.id,
            //                 &interaction.token,
            //                 &InteractionResponse::DeferredChannelMessageWithSource(CallbackData {
            //                     allowed_mentions: None,
            //                     components: None,
            //                     content: None,
            //                     embeds: Default::default(),
            //                     flags: None,
            //                     tts: None,
            //                 }),
            //             )
            //             .exec()
            //             .await
            //         {
            //             error!(%err, "failed sending interaction response");
            //         }
            //     }
            //     self.send_discord_guild_event(evt).await;
            // }
            _ => {
                self.send_discord_guild_event(evt).await;
            }
        }
    }

    async fn send_discord_guild_event(&mut self, evt: GuildEvent) {
        if let Some(evt) = crate::dispatch_conv::discord_event_to_dispatch(*evt.event) {
            self.dispatch_worker_evt(evt.name.to_string(), evt.data, PendingAck::Dispatch(None))
                .await;
        }
    }

    async fn dispatch_worker_evt(&mut self, t: String, data: serde_json::Value, ack: PendingAck) {
        if self.scripts.is_empty() {
            return;
        }

        loop {
            self.ensure_claim_worker().await;

            let evt_id = self.gen_id();

            if let Some(worker) = &self.current_worker {
                match worker.tx.send(SchedulerMessage::Dispatch(VmDispatchEvent {
                    name: t.clone(),
                    seq: evt_id,
                    value: data.clone(),
                })) {
                    Ok(_) => {
                        self.pending_acks.insert(evt_id, ack);
                        return;
                    }
                    Err(_) => {
                        error!("worker died while trying to dispatch event, retrying in a second");
                        self.broken_worker().await;
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }

    async fn ensure_claim_worker(&mut self) {
        if self.current_worker.is_none() {
            loop {
                let mut worker = self.worker_pool.req_worker(self.guild_id).await;

                #[allow(clippy::collapsible_if)]
                if self.should_send_scripts(&worker) {
                    self.pending_acks.clear();
                    self.reset_contribs();

                    if worker
                        .tx
                        .send(SchedulerMessage::UpdateRunState(
                            self.gen_id(),
                            UpdateRunStateRequest {
                                guild_id: self.guild_id,
                                guild_scripts: RunStateChangeReq::Start(self.scripts.clone()),
                                packs: RunStateChangeReq::Stop,
                            },
                        ))
                        .is_err()
                    {
                        worker.last_active_guild = Some(self.guild_id);

                        // broken worker, get a new one
                        self.worker_pool.return_worker(worker, true);
                        continue;
                    }
                }

                worker.last_active_guild = Some(self.guild_id);
                self.force_load_scripts_next = false;
                self.current_worker = Some(worker);

                // new worker, clear pending acks, should trigger retries for most things
                info!("claimed new worker");
                break;
            }
        }
    }

    async fn broken_worker(&mut self) {
        if let Some(mut worker) = self.current_worker.take() {
            while let Ok(msg) = worker.rx.try_recv() {
                self.handle_worker_msg(msg).await;
            }

            self.worker_pool.return_worker(worker, true);
            self.reset_contribs();
            self.pending_acks.clear();
        }
    }

    fn reset_contribs(&mut self) {
        self.interval_timers_man.clear_loaded_timers();
        self.interval_timers_man.clear_pending_acks();
        self.scheduled_tasks_man.clear_pending();
        self.scheduled_tasks_man.clear_task_names();
        self.scheduled_tasks_man.clear_next();
    }

    fn should_send_scripts(&mut self, worker: &WorkerHandle) -> bool {
        if !self.force_load_scripts_next {
            if let Some(guild_id) = worker.last_active_guild {
                if guild_id == self.guild_id {
                    return false;
                }
            }
        }

        true
    }

    fn gen_id(&mut self) -> u64 {
        self.id_gen += 1;
        self.id_gen
    }

    async fn script_loaded(&mut self, evt: ScriptMeta) {
        let interval_contribs: Vec<IntervalTimerContrib> = evt
            .interval_timers
            .iter()
            .map(|v| stores::config::IntervalTimerContrib {
                name: v.name.clone(),
                interval: match &v.interval {
                    runtime_models::internal::script::IntervalType::Cron(c) => {
                        stores::timers::IntervalType::Cron(c.clone())
                    }
                    runtime_models::internal::script::IntervalType::Minutes(m) => {
                        stores::timers::IntervalType::Minutes(m.0)
                    }
                },
            })
            .collect();

        self.update_db_contribs(&evt, interval_contribs.clone())
            .await;

        self.interval_timers_man
            .script_started(interval_contribs)
            .await;

        self.scheduled_tasks_man.script_started(&evt);

        self.cmd_manager_handle.send(command_manager::LoadedScript {
            guild_id: self.guild_id,
            meta: evt,
        });
    }

    async fn update_db_contribs(
        &mut self,
        evt: &ScriptMeta,
        interval_contribs: Vec<IntervalTimerContrib>,
    ) {
        let twilight_commands = crate::command_manager::to_twilight_commands(
            self.guild_id,
            &evt.commands,
            &evt.command_groups,
        );

        // TODO: handle errors here, maybe retry?
        if let Err(err) = self
            .stores
            .update_script_contributes(
                self.guild_id,
                evt.script_id.0,
                ScriptContributes {
                    commands: twilight_commands,
                    interval_timers: interval_contribs,
                },
            )
            .await
        {
            error!(%err, "failed updating db contribs");
        }
    }
}

enum NextGuildAction {
    TimerCheck,
    GuildCommand(GuildCommand),
    WorkerMessage(Option<WorkerMessage>),
    DispatchScheduledTasks(Vec<ScheduledTask>),
    DispatchIntervalTimers(Vec<IntervalTimer>),
}

struct NextGuildHandlerActionFuture<'a> {
    guild_handler: &'a mut GuildHandler,
}

impl Future for NextGuildHandlerActionFuture<'_> {
    type Output = Option<NextGuildAction>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.guild_handler.guild_rx.poll_recv(cx) {
            Poll::Ready(Some(evt)) => return Poll::Ready(Some(NextGuildAction::GuildCommand(evt))),
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Pending => {}
        };

        if let Some(worker) = &mut self.guild_handler.current_worker {
            if let Poll::Ready(r) = worker.rx.poll_recv(cx) {
                return Poll::Ready(Some(NextGuildAction::WorkerMessage(r)));
            }
        }

        Poll::Pending
    }
}

pub enum GuildHandlerEvent {
    ForciblyShutdown,
}

pub struct GuildHandle {
    pub guild_id: Id<GuildMarker>,
    pub rx: mpsc::UnboundedReceiver<GuildHandlerEvent>,
    pub tx: Option<mpsc::UnboundedSender<GuildCommand>>,
}

pub enum PendingAck {
    Dispatch(Option<oneshot::Sender<()>>),
    ScheduledTask(u64),
    IntervalTimer(String),
}

pub enum NextTimerAction<T> {
    None,
    Wait(DateTime<Utc>),
    Run(T),
}

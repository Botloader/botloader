use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};

use crate::{
    command_manager,
    guild_handler::PremiumTierState,
    interval_timer_manager, scheduled_task_manager,
    scheduler::Store,
    vmworkerpool::{WorkerHandle, WorkerRetrieved},
};
use common::DiscordConfig;
use dbrokerapi::broker_scheduler_rpc::GuildEvent;
use guild_logger::{GuildLogger, LogEntry};
use runtime_models::internal::script::ScriptMeta;
use scheduler_worker_rpc::{
    CreateScriptsVmReq, MetricEvent, SchedulerMessage, VmDispatchEvent, WorkerMessage,
};
use stores::{
    config::{IntervalTimerContrib, Script, ScriptContributes},
    plugins::{GuildPluginSubscription, Plugin, Version},
    timers::{IntervalTimer, ScheduledTask},
};
use tokio::sync::oneshot;
use tracing::{error, info, instrument};
use twilight_model::id::{marker::GuildMarker, Id};

pub struct VmSession {
    guild_id: Id<GuildMarker>,

    discord_config: Arc<DiscordConfig>,
    stores: Arc<dyn Store>,
    logger: GuildLogger,
    worker_pool: crate::vmworkerpool::VmWorkerPool,
    interval_timers_man: crate::interval_timer_manager::Manager,
    cmd_manager_handle: command_manager::Handle,
    scheduled_tasks_man: scheduled_task_manager::Manager,

    premium_tier: Arc<RwLock<PremiumTierState>>,

    vm_type: SessionType,

    pending_acks: HashMap<u64, PendingAck>,
    current_worker: Option<WorkerHandle>,
    force_load_scripts_next: bool,
    id_gen: u64,
}

impl VmSession {
    pub async fn new(
        kind: CreateSessionType,
        stores: Arc<dyn Store>,
        guild_id: Id<GuildMarker>,
        logger: GuildLogger,
        worker_pool: crate::vmworkerpool::VmWorkerPool,
        cmd_manager_handle: crate::command_manager::Handle,
        discord_config: Arc<DiscordConfig>,
        premium_tier: Arc<RwLock<PremiumTierState>>,
    ) -> Result<VmSession, anyhow::Error> {
        let loaded_conf = kind.load(guild_id, &stores).await?;

        let interval_timer_man =
            crate::interval_timer_manager::Manager::new(guild_id, stores.clone());

        let tasks_man = scheduled_task_manager::Manager::new(guild_id, stores.clone());

        Ok(VmSession {
            stores,
            guild_id,
            logger,
            worker_pool,
            discord_config,
            premium_tier,
            vm_type: loaded_conf,

            id_gen: 1,
            pending_acks: HashMap::new(),
            current_worker: None,
            force_load_scripts_next: false,

            interval_timers_man: interval_timer_man,
            cmd_manager_handle,
            scheduled_tasks_man: tasks_man,
        })
    }

    pub async fn start(&mut self) {
        self.load_contribs().await;
    }

    #[instrument(skip(self, action), fields(guild_id = self.guild_id.get()))]
    pub async fn handle_action(&mut self, action: NextAction) -> Option<VmSessionEvent> {
        match action {
            NextAction::WorkerMessage(Some(WorkerMessage::Shutdown(reason))) => {
                self.logger.log(LogEntry::critical(
                    self.guild_id,
                    format!("vm was forcibly shut down, reason: {:?}", reason),
                ));

                self.reset_contribs();
                self.pending_acks.clear();

                match reason {
                    scheduler_worker_rpc::ShutdownReason::TooManyInvalidRequests => {
                        return Some(VmSessionEvent::TooManyInvalidRequests);
                    }
                    _ => {
                        return Some(VmSessionEvent::ForciblyShutdown);
                    }
                }
            }
            NextAction::WorkerMessage(Some(msg)) => {
                self.handle_worker_msg(msg).await;
            }
            NextAction::WorkerMessage(None) => {
                self.broken_worker().await;
            }
            NextAction::CheckScheduledTasks => {
                let tasks = self.scheduled_tasks_man.start_triggered_tasks().await;
                for task in tasks {
                    self.dispatch_scheduled_task(task).await;
                }
            }
            NextAction::CheckIntervalTimers => {
                let timers = self.interval_timers_man.trigger_timers().await;
                for timer in timers {
                    self.dispatch_interval_timer(timer).await;
                }
            }
        }

        None
    }

    pub async fn shutdown(&mut self) {
        info!("shutting down vm session");

        // wait until the vm has finished it's work
        if let Some(worker) = &mut self.current_worker {
            if worker.tx.send(SchedulerMessage::Complete).is_err() {
                self.broken_worker().await;
            };
        }

        loop {
            if self.current_worker.is_none() {
                break;
            }

            match self.next_action().await {
                NextAction::WorkerMessage(Some(msg)) => self.handle_worker_msg(msg).await,
                NextAction::WorkerMessage(None) => {
                    self.broken_worker().await;
                    break;
                }
                NextAction::CheckScheduledTasks => {}
                NextAction::CheckIntervalTimers => {}
            }
        }
    }

    fn get_premium_tier(&self) -> PremiumTierState {
        let r = self.premium_tier.read().unwrap();
        *r
    }

    pub async fn load_contribs(&mut self) {
        info!("loading contribs");

        let evt_id = self.gen_id();
        if let Some(worker) = &self.current_worker {
            let req = self.make_create_scripts_req(evt_id);
            if worker.tx.send(req).is_err() {
                self.broken_worker().await;
            }
        } else {
            self.reset_contribs();

            if self.vm_type.is_scripts_empty() {
                return;
            }

            self.force_load_scripts_next = true;
            self.ensure_claim_worker().await;
        }
    }

    pub async fn next_action(&mut self) -> NextAction {
        let scheduled_task_sleep_check = match self.scheduled_tasks_man.next_action() {
            scheduled_task_manager::NextAction::None => {
                tokio::time::sleep(Duration::from_secs(60 * 60))
            }
            scheduled_task_manager::NextAction::Wait(until) => {
                let sleep_dur = (until - chrono::Utc::now())
                    .to_std()
                    .unwrap_or_else(|_| Duration::from_millis(1));
                tokio::time::sleep(sleep_dur)
            }
            scheduled_task_manager::NextAction::Run => {
                return NextAction::CheckScheduledTasks;
            }
        };

        let interval_timers_sleep_check = match self.interval_timers_man.next_action() {
            interval_timer_manager::NextAction::None => {
                tokio::time::sleep(Duration::from_secs(60 * 60))
            }
            interval_timer_manager::NextAction::Wait(until) => {
                let sleep_dur = (until - chrono::Utc::now())
                    .to_std()
                    .unwrap_or_else(|_| Duration::from_millis(1));
                tokio::time::sleep(sleep_dur)
            }
            interval_timer_manager::NextAction::Run => {
                return NextAction::CheckIntervalTimers;
            }
        };

        tokio::pin!(scheduled_task_sleep_check);
        tokio::pin!(interval_timers_sleep_check);

        if let Some(worker) = &mut self.current_worker {
            tokio::select! {
                evt = worker.rx.recv() =>{
                    NextAction::WorkerMessage(evt)
                },
                _ = scheduled_task_sleep_check => {
                    NextAction::CheckScheduledTasks
                },
                _ = interval_timers_sleep_check => {
                    NextAction::CheckIntervalTimers
                }
            }
        } else {
            tokio::select! {
                _ = scheduled_task_sleep_check => {
                    NextAction::CheckScheduledTasks
                },
                _ = interval_timers_sleep_check => {
                    NextAction::CheckIntervalTimers
                }
            }
        }
    }

    pub async fn init_timers(&mut self) {
        self.scheduled_tasks_man.init_next_task_time().await;
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

    pub async fn reload_scripts(&mut self) {
        match self.vm_type.load(self.guild_id, &self.stores).await {
            Ok(new_conf) => {
                self.vm_type = new_conf;
                self.load_contribs().await;
            }
            Err(err) => error!(%err, "failed reloading vm session config"),
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

    pub async fn send_discord_guild_event(&mut self, evt: GuildEvent) {
        if let Some(evt) = crate::dispatch_conv::discord_event_to_dispatch(*evt.event) {
            self.dispatch_worker_evt(evt.name.to_string(), evt.data, PendingAck::Dispatch(None))
                .await;
        }
    }

    async fn dispatch_worker_evt(&mut self, t: String, data: serde_json::Value, ack: PendingAck) {
        if self.vm_type.is_scripts_empty() {
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
                let (worker, wr) = self
                    .worker_pool
                    .req_worker(self.guild_id, self.get_premium_tier().option())
                    .await;

                #[allow(clippy::collapsible_if)]
                if self.should_send_scripts(wr) {
                    self.pending_acks.clear();
                    self.reset_contribs();

                    let seq = self.gen_id();
                    if worker.tx.send(self.make_create_scripts_req(seq)).is_err() {
                        // broken worker, get a new one
                        self.worker_pool.return_worker(worker, true);
                        continue;
                    }
                }

                info!(tier = worker.priority_index, "claimed new worker");
                self.force_load_scripts_next = false;
                self.current_worker = Some(worker);
                break;
            }
        }
    }

    fn make_create_scripts_req(&self, seq: u64) -> SchedulerMessage {
        SchedulerMessage::CreateScriptsVm(CreateScriptsVmReq {
            seq,
            guild_id: self.guild_id,
            premium_tier: self.get_premium_tier().option(),
            scripts: match &self.vm_type {
                SessionType::GuildScripts(data) => data.scripts.clone(),
                SessionType::Plugin(_) => todo!(),
            },
        })
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

    fn should_send_scripts(&mut self, wr: WorkerRetrieved) -> bool {
        if !self.force_load_scripts_next && matches!(wr, WorkerRetrieved::SameGuild) {
            return false;
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

pub enum NextAction {
    WorkerMessage(Option<WorkerMessage>),
    CheckScheduledTasks,
    CheckIntervalTimers,
}

pub enum VmSessionEvent {
    TooManyInvalidRequests,
    ForciblyShutdown,
}

pub enum PendingAck {
    Dispatch(Option<oneshot::Sender<()>>),
    ScheduledTask(u64),
    IntervalTimer(String),
}

pub enum CreateSessionType {
    GuildScripts,
    Plugin(u64),
}

impl CreateSessionType {
    async fn load(
        &self,
        guild_id: Id<GuildMarker>,
        stores: &Arc<dyn Store>,
    ) -> Result<SessionType, anyhow::Error> {
        match self {
            Self::GuildScripts => {
                Ok(SessionType::try_retry_load_guild_scripts(guild_id, stores).await)
            }
            Self::Plugin(id) => SessionType::load_plugin(guild_id, *id, stores).await,
        }
    }
}

pub enum SessionType {
    GuildScripts(GuildScriptsSession),
    Plugin(PluginSession),
}

pub struct GuildScriptsSession {
    scripts: Vec<Script>,
}

pub struct PluginSession {
    plugin: Plugin,
    subscription: GuildPluginSubscription,
    loaded_version: Version,
}

impl SessionType {
    async fn try_retry_load_guild_scripts(
        guild_id: Id<GuildMarker>,
        stores: &Arc<dyn Store>,
    ) -> SessionType {
        loop {
            match stores.list_scripts(guild_id).await {
                Ok(scripts) => {
                    let scripts = scripts.into_iter().filter(|v| v.enabled).collect();
                    return Self::GuildScripts(GuildScriptsSession { scripts });
                }
                Err(err) => {
                    error!(%err, "failed loading guild scripts, retrying in 10 secs");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            }
        }
    }

    async fn load_plugin(
        guild_id: Id<GuildMarker>,
        plugin_id: u64,
        stores: &Arc<dyn Store>,
    ) -> Result<SessionType, anyhow::Error> {
        todo!();
    }

    async fn load(
        &self,
        guild_id: Id<GuildMarker>,
        stores: &Arc<dyn Store>,
    ) -> Result<Self, anyhow::Error> {
        match self {
            Self::GuildScripts(_) => Ok(Self::try_retry_load_guild_scripts(guild_id, stores).await),
            Self::Plugin(plugin) => Self::load_plugin(guild_id, plugin.plugin.id, stores).await,
        }
    }

    fn is_scripts_empty(&self) -> bool {
        match self {
            Self::GuildScripts(data) => data.scripts.is_empty(),
            Self::Plugin(data) => data.loaded_version.data.sources.is_empty(),
        }
    }
}

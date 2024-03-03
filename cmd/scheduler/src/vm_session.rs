use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use crate::{
    command_manager,
    guild_handler::PremiumTierState,
    interval_timer_manager::{self, TimerId},
    scheduled_task_manager,
    scheduler::Store,
    vmworkerpool::{WorkerHandle, WorkerRetrieved},
};
use common::DiscordConfig;
use dbrokerapi::broker_scheduler_rpc::DiscordEvent;
use guild_logger::{entry::CreateLogEntry, GuildLogSender};
use runtime_models::{internal::script::ScriptMeta, util::PluginId};
use scheduler_worker_rpc::{
    CreateScriptsVmReq, MetricEvent, SchedulerMessage, VmDispatchEvent, WorkerMessage,
};
use stores::{
    config::{IntervalTimerContrib, Script, ScriptContributes},
    timers::{IntervalTimer, ScheduledTask},
};
use tokio::sync::oneshot;
use tracing::{error, info, instrument};
use twilight_model::id::{marker::GuildMarker, Id};

pub struct VmSession {
    guild_id: Id<GuildMarker>,

    _discord_config: Arc<DiscordConfig>,
    stores: Arc<dyn Store>,
    logger: GuildLogSender,
    worker_pool: crate::vmworkerpool::VmWorkerPool,
    interval_timers_man: crate::interval_timer_manager::Manager,
    cmd_manager_handle: command_manager::Handle,
    scheduled_tasks_man: scheduled_task_manager::Manager,

    premium_tier: Arc<RwLock<PremiumTierState>>,

    pending_acks: HashMap<u64, PendingAck>,
    current_worker: Option<WorkerHandle>,
    force_load_scripts_next: bool,
    scripts: Vec<Script>,
    id_gen: u64,

    last_claimed_worker_id: Option<u64>,
    last_claimed_worker_at: Instant,
    last_returned_worker_at: Instant,
}

impl VmSession {
    pub fn new(
        stores: Arc<dyn Store>,
        guild_id: Id<GuildMarker>,
        logger: GuildLogSender,
        worker_pool: crate::vmworkerpool::VmWorkerPool,
        cmd_manager_handle: crate::command_manager::Handle,
        discord_config: Arc<DiscordConfig>,
        premium_tier: Arc<RwLock<PremiumTierState>>,
    ) -> VmSession {
        let interval_timer_man =
            crate::interval_timer_manager::Manager::new(guild_id, stores.clone());

        let tasks_man = scheduled_task_manager::Manager::new(guild_id, stores.clone());

        VmSession {
            stores,
            guild_id,
            logger,
            worker_pool,
            _discord_config: discord_config,
            premium_tier,

            id_gen: 1,
            pending_acks: HashMap::new(),
            current_worker: None,
            scripts: Vec::new(),
            force_load_scripts_next: false,

            interval_timers_man: interval_timer_man,
            cmd_manager_handle,
            scheduled_tasks_man: tasks_man,

            last_claimed_worker_id: None,
            last_claimed_worker_at: Instant::now(),
            last_returned_worker_at: Instant::now(),
        }
    }

    pub fn _set_guild_scripts(&mut self, scripts: Vec<Script>) {
        self.scripts = scripts;
        self.force_load_scripts_next = true;
    }

    pub async fn start(&mut self) {
        self.try_retry_load_guild_scripts().await;
        self.load_contribs().await;
    }

    pub fn get_status(&self) -> VmSessionStatus {
        VmSessionStatus {
            current_claimed_worker: self.current_worker.as_ref().map(|v| v.worker_id),
            claimed_worker_at: self.last_claimed_worker_at,
            returned_worker_at: self.last_returned_worker_at,
            last_claimed_worker: self.last_claimed_worker_id,
            num_pending_acks: self.pending_acks.len(),
        }
    }

    #[instrument(skip(self, action), fields(guild_id = self.guild_id.get()))]
    pub async fn handle_action(&mut self, action: NextAction) -> Option<VmSessionEvent> {
        match action {
            NextAction::WorkerMessage(Some(WorkerMessage::Shutdown(reason))) => {
                self.logger.log(CreateLogEntry::critical(format!(
                    "vm was forcibly shut down, reason: {reason:?}"
                )));

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
                // Complicated logic to avoid runaway workers and duplicated runs
                // TODO: should add some proper protection for duplicated runs to simplify this
                if self.current_worker.is_some() {
                    let tasks = self.scheduled_tasks_man.start_triggered_tasks().await;
                    for task in tasks {
                        self.dispatch_scheduled_task(task).await;
                    }
                } else {
                    match self.ensure_claim_worker().await {
                        ClaimWorkerResult::Reused => {
                            let tasks = self.scheduled_tasks_man.start_triggered_tasks().await;
                            if tasks.is_empty() {
                                if self.pending_acks.is_empty() {
                                    self.return_worker();
                                }
                            } else {
                                for task in tasks {
                                    self.dispatch_scheduled_task(task).await;
                                }
                            }
                        }
                        ClaimWorkerResult::Reloaded => {
                            // contribs was cleared, no tasks/timers to fire.
                        }
                    }
                }
            }
            NextAction::CheckIntervalTimers => {
                // Complicated logic to avoid runaway workers and duplicated runs
                // TODO: should add some proper protection for duplicated runs to simplify this
                if self.current_worker.is_some() {
                    let timers = self.interval_timers_man.trigger_timers().await;
                    for timer in timers {
                        self.dispatch_interval_timer(timer).await;
                    }
                } else {
                    match self.ensure_claim_worker().await {
                        ClaimWorkerResult::Reused => {
                            let timers = self.interval_timers_man.trigger_timers().await;
                            if timers.is_empty() {
                                if self.pending_acks.is_empty() {
                                    self.return_worker();
                                }
                            } else {
                                for timer in timers {
                                    self.dispatch_interval_timer(timer).await;
                                }
                            }
                        }
                        ClaimWorkerResult::Reloaded => {
                            // contribs was cleared, no tasks/timers to fire.
                        }
                    }
                }
            }
        }

        None
    }

    fn return_worker(&mut self) {
        if let Some(current) = self.current_worker.take() {
            self.last_claimed_worker_id = Some(current.worker_id);
            self.last_returned_worker_at = Instant::now();

            self.worker_pool.return_worker(current, false);
        }
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

        if self.scripts.is_empty() {
            self.cmd_manager_handle
                .send_no_scripts_enabled(self.guild_id);
        }

        if self.current_worker.is_some() {
            if self.send_create_scripts_vm().await.is_err() {
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

    async fn try_retry_load_guild_scripts(&mut self) {
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
                        PendingAck::IntervalTimer(timer) => {
                            self.interval_timers_man.timer_ack(&timer).await;
                        }
                        PendingAck::Restart => {}
                    }
                }
            }
            WorkerMessage::ScriptStarted(start) => {
                self.script_loaded(start).await;
            }
            WorkerMessage::ScriptsInit => todo!(),
            WorkerMessage::NonePending => {
                if self.pending_acks.is_empty() {
                    self.return_worker();
                }
            }
            WorkerMessage::TaskScheduled => {
                self.scheduled_tasks_man.clear_next();
            }
            WorkerMessage::GuildLog(entry) => {
                self.logger.log_raw(entry);
            }
            WorkerMessage::Hello(_) => {
                // handled when connection is established, not applicable here
                unreachable!();
            }
            WorkerMessage::Shutdown(_) => {
                // handled in caller
            }
            WorkerMessage::Metric(name, m, labels) => self.handle_metric(name, m, labels),
        }
    }

    fn handle_metric(&mut self, name: String, m: MetricEvent, labels: HashMap<String, String>) {
        let mut labels = labels
            .into_iter()
            .map(|(k, v)| metrics::Label::new(k, v))
            .collect::<Vec<_>>();

        labels.push(metrics::Label::new("guild_id", self.guild_id.to_string()));

        match m {
            MetricEvent::Gauge(action) => {
                let handle = metrics::gauge!(name, labels);
                match action {
                    scheduler_worker_rpc::GaugeEvent::Set(v) => handle.set(v),
                    scheduler_worker_rpc::GaugeEvent::Incr(v) => handle.increment(v),
                }
            }
            MetricEvent::Counter(action) => {
                let handle = metrics::counter!(name, labels);

                match action {
                    scheduler_worker_rpc::CounterEvent::Incr(v) => handle.increment(v),
                    scheduler_worker_rpc::CounterEvent::Absolute(v) => handle.absolute(v),
                }
            }
        }
    }

    pub async fn reload_guild_scripts(&mut self) {
        self.try_retry_load_guild_scripts().await;
        self.load_contribs().await;
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
            plugin_id: timer.plugin_id.map(PluginId),
        };

        let serialized = serde_json::to_value(&evt).unwrap();
        self.dispatch_worker_evt(
            "BOTLOADER_INTERVAL_TIMER_FIRED".to_string(),
            serialized,
            PendingAck::IntervalTimer(TimerId::new(timer.plugin_id, timer.name)),
        )
        .await;
    }

    pub async fn send_discord_guild_event(&mut self, evt: DiscordEvent) {
        let t_clone = evt.t.clone();
        match crate::dispatch_conv::discord_event_to_dispatch(evt) {
            Ok(Some(converted_evt)) => {
                self.dispatch_worker_evt(
                    converted_evt.name.to_string(),
                    converted_evt.data,
                    PendingAck::Dispatch(None),
                )
                .await;
            }
            Ok(None) => {
                tracing::warn!(t = t_clone, "skipped converting dispatch event")
            }
            Err(err) => {
                error!(%err, t=t_clone, "failed converting dispatch event")
            }
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

    async fn ensure_claim_worker(&mut self) -> ClaimWorkerResult {
        if self.current_worker.is_some() {
            return ClaimWorkerResult::Reused;
        }

        loop {
            let (worker, wr) = self
                .worker_pool
                .req_worker(self.guild_id, self.get_premium_tier().option())
                .await;

            let should_create_vm = self.should_send_scripts(wr);

            info!(tier = worker.priority_index, "claimed new worker");
            self.current_worker = Some(worker);
            self.last_claimed_worker_at = Instant::now();

            let res = if should_create_vm {
                // new worker, reset acks and whatnot
                self.pending_acks.clear();
                self.reset_contribs();
                if self.send_create_scripts_vm().await.is_err() {
                    self.broken_worker().await;
                    // try again
                    continue;
                }
                ClaimWorkerResult::Reloaded
            } else {
                ClaimWorkerResult::Reused
            };

            self.force_load_scripts_next = false;
            break res;
        }
    }

    // TODO: this should reset contribs in some way, reason for this being
    // task handlers/interval timers could have been removed
    //
    // but we would have to wait for potential in flight tasks/timers
    // and the only downside for not resetting contribs is timers being fired that's not used
    async fn send_create_scripts_vm(&mut self) -> Result<(), ()> {
        let evt_id = self.gen_id();

        if let Some(worker) = &self.current_worker {
            if worker
                .tx
                .send(SchedulerMessage::CreateScriptsVm(CreateScriptsVmReq {
                    seq: evt_id,
                    guild_id: self.guild_id,
                    premium_tier: self.get_premium_tier().option(),
                    scripts: self.scripts.clone(),
                }))
                .is_err()
            {
                return Err(());
            }

            self.pending_acks.insert(evt_id, PendingAck::Restart);
        } else {
            panic!("no worker");
        }

        Ok(())
    }

    async fn broken_worker(&mut self) {
        if let Some(mut worker) = self.current_worker.take() {
            self.last_claimed_worker_id = Some(worker.worker_id);
            self.last_returned_worker_at = Instant::now();

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
                plugin_id: evt.plugin_id.map(|v| v.0),
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

        self.cmd_manager_handle
            .send_loaded_script(self.guild_id, evt);
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
    IntervalTimer(TimerId),
    Restart,
}

enum ClaimWorkerResult {
    // A worker was already claimed or we managed to reuse the worker from the pool we had last time
    Reused,
    // We had to reload our VM
    Reloaded,
}

pub struct VmSessionStatus {
    pub current_claimed_worker: Option<u64>,
    pub last_claimed_worker: Option<u64>,
    pub claimed_worker_at: Instant,
    pub returned_worker_at: Instant,
    pub num_pending_acks: usize,
}

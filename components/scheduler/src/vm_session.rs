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
    vmworkerpool::{WorkerHandle, WorkerRetrieved},
    SchedulerConfig,
};
use chrono::{DateTime, Utc};
use common::dispatch_event::{EventSource, VmDispatchEvent};
use dbrokerapi::broker_scheduler_rpc::DiscordEvent;
use guild_logger::{entry::CreateLogEntry, GuildLogSender};
use runtime_models::{internal::script::ScriptMeta, util::PluginId};
use scheduler_worker_rpc::{CreateScriptsVmReq, MetricEvent, SchedulerMessage, WorkerMessage};
use stores::{
    config::{IntervalTimerContrib, Script, ScriptContributes, UpdateScript},
    timers::{IntervalTimer, ScheduledTask},
    Db,
};
use tokio::sync::oneshot;
use tracing::{error, info, instrument, warn};
use twilight_model::id::{marker::GuildMarker, Id};
use vm::vm::ShutdownReason;

pub struct VmSession {
    guild_id: Id<GuildMarker>,

    config: Arc<SchedulerConfig>,
    stores: Db,
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

    dispatch_id_gen: u64,
    current_vm_session_id: u64,

    last_claimed_worker_id: Option<u64>,
    last_claimed_worker_at: Instant,
    last_returned_worker_at: Instant,
}

impl VmSession {
    pub fn new(
        config: Arc<SchedulerConfig>,
        stores: Db,
        guild_id: Id<GuildMarker>,
        logger: GuildLogSender,
        worker_pool: crate::vmworkerpool::VmWorkerPool,
        cmd_manager_handle: crate::command_manager::Handle,
        premium_tier: Arc<RwLock<PremiumTierState>>,
    ) -> VmSession {
        let interval_timer_man =
            crate::interval_timer_manager::Manager::new(guild_id, stores.clone());

        let tasks_man = scheduled_task_manager::Manager::new(guild_id, stores.clone());

        VmSession {
            config,
            stores,
            guild_id,
            logger,
            worker_pool,
            premium_tier,
            dispatch_id_gen: 1,
            current_vm_session_id: 1,
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
        self.start_fresh_vm().await;
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
            NextAction::WorkerMessage(Some(WorkerMessage::Shutdown(shutdown))) => {
                if shutdown.guild_id != self.guild_id {
                    // not relevant to our guild
                    return None;
                }

                self.cancel_old_vm_session_pending_acks(shutdown.vm_session_id);

                if self.current_vm_session_id == shutdown.vm_session_id {
                    // return to pool and reset
                    self.force_load_scripts_next = true;
                    self.return_worker();
                } else {
                    // we will be receiving a new set of tasks and timers for the new vm
                    self.scheduled_tasks_man.clear_task_names();
                    self.interval_timers_man.clear_loaded_timers();
                }

                match shutdown.reason {
                    Some(ShutdownReason::DiscordInvalidRequestsRatelimit) => {
                        self.logger.log(CreateLogEntry::critical(
                            "VM was forcibly shut down for issuing too many invalid discord \
                             requests. Your guild has also been temporarily blacklisted for this \
                             reason."
                                .to_string(),
                        ));

                        return Some(VmSessionEvent::ShutdownTooManyInvalidRequests);
                    }
                    Some(ShutdownReason::Runaway) => {
                        self.logger.log(CreateLogEntry::critical(
                            "VM was forcibly shut down for blocking the thread for too long, this \
                             means you might have an infinite loop somewhere."
                                .to_string(),
                        ));

                        return Some(VmSessionEvent::ShutdownExcessCpu);
                    }
                    Some(ShutdownReason::OutOfMemory) => {
                        self.logger.log(CreateLogEntry::critical(
                            "VM ran out of memory, this should not normally happen from normal \
                             use but if this was not intentional abuse please join the support \
                             server and let us know."
                                .to_owned(),
                        ));
                        warn!("VM ran out of memory");
                    }
                    Some(ShutdownReason::Request) => {
                        self.logger.log(CreateLogEntry::info(
                            "Vm was shut down gracefully.".to_owned(),
                        ));
                    }
                    None => {
                        error!("Unknown vm shutdown reason");
                        self.logger.log(CreateLogEntry::error(
                            "Vm was shut down due to a unknown reason?".to_owned(),
                        ));
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
        };

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
                NextAction::WorkerMessage(Some(WorkerMessage::Shutdown(evt))) => {
                    info!("Got shutdown event form vm! reason: {:?}", evt.reason);

                    if evt.vm_session_id == self.current_vm_session_id {
                        self.return_worker();
                        self.force_load_scripts_next = true;
                        break;
                    }
                }
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

    pub async fn start_fresh_vm(&mut self) {
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
            // if we have not claimed a worker then there's no pending acks
            self.clear_loaded_timers_and_tasks();

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
            scheduled_task_manager::NextAction::None => tokio::time::sleep(Duration::MAX),
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
            interval_timer_manager::NextAction::None => tokio::time::sleep(Duration::MAX),
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
                    match item.kind {
                        PendingAckType::Dispatch(Some(resp)) => {
                            let _ = resp.send(());
                        }
                        PendingAckType::Dispatch(_) => {}
                        PendingAckType::ScheduledTask(t_id) => {
                            self.scheduled_tasks_man.ack_triggered_task(t_id).await;
                        }
                        PendingAckType::IntervalTimer(timer) => {
                            self.interval_timers_man.timer_ack(&timer).await;
                        }
                        PendingAckType::Restart => {}
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

        if name != "dispatch_event_latency" {
            labels.push(metrics::Label::new("guild_id", self.guild_id.to_string()));
        }

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
            MetricEvent::Histogram(action) => {
                metrics::histogram!(name, labels).record(action);
            }
        }
    }

    pub async fn reload_guild_scripts(&mut self) {
        self.try_retry_load_guild_scripts().await;
        self.start_fresh_vm().await;
    }

    async fn dispatch_scheduled_task(&mut self, task: ScheduledTask) {
        info!("dispatching scheduled task");
        let task_id = task.id;
        let evt = runtime_models::internal::tasks::ScheduledTask::from(task);
        let serialized = serde_json::to_value(&evt).unwrap();
        self.dispatch_worker_evt(
            "BOTLOADER_SCHEDULED_TASK_FIRED".to_string(),
            serialized,
            PendingAckType::ScheduledTask(task_id),
            EventSource::Timer,
            Utc::now(),
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
            PendingAckType::IntervalTimer(TimerId::new(timer.plugin_id, timer.name)),
            EventSource::Timer,
            Utc::now(),
        )
        .await;
    }

    pub async fn send_discord_guild_event(&mut self, evt: DiscordEvent) {
        let t_clone = evt.t.clone();
        let ts_clone = evt.timestamp;
        match crate::dispatch_conv::discord_event_to_dispatch(evt) {
            Ok(Some(converted_evt)) => {
                self.dispatch_worker_evt(
                    converted_evt.name.to_string(),
                    converted_evt.data,
                    PendingAckType::Dispatch(None),
                    EventSource::Discord,
                    ts_clone,
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

    async fn dispatch_worker_evt(
        &mut self,
        t: String,
        data: serde_json::Value,
        ack: PendingAckType,
        source: EventSource,
        ts: DateTime<Utc>,
    ) {
        if self.scripts.is_empty() {
            return;
        }

        loop {
            self.ensure_claim_worker().await;

            let evt_id = self.gen_dispatch_id();

            if let Some(worker) = &self.current_worker {
                match worker.tx.send(SchedulerMessage::Dispatch(VmDispatchEvent {
                    name: t.clone(),
                    seq: evt_id,
                    value: data.clone(),
                    source,
                    source_timestamp: ts,
                })) {
                    Ok(_) => {
                        self.pending_acks.insert(
                            evt_id,
                            PendingAck {
                                dispatched_session_id: self.current_vm_session_id,
                                kind: ack,
                            },
                        );
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

    #[instrument(skip_all)]
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
                // new vm, we will receive new timers
                self.clear_loaded_timers_and_tasks();

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

    #[instrument(skip_all)]
    async fn send_create_scripts_vm(&mut self) -> Result<(), ()> {
        let evt_id = self.gen_dispatch_id();
        let new_session_id = self.invalidate_create_new_session_id();

        if let Some(worker) = &self.current_worker {
            if worker
                .tx
                .send(SchedulerMessage::CreateScriptsVm(CreateScriptsVmReq {
                    seq: evt_id,
                    session_id: new_session_id,
                    guild_id: self.guild_id,
                    premium_tier: self.get_premium_tier().option(),
                    scripts: self.scripts.clone(),
                }))
                .is_err()
            {
                return Err(());
            }

            self.pending_acks.insert(
                evt_id,
                PendingAck {
                    dispatched_session_id: self.current_vm_session_id,
                    kind: PendingAckType::Restart,
                },
            );
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
            self.clear_loaded_timers_and_tasks();
            self.clear_all_pending_timer_acks();
            self.pending_acks.clear();

            // We could start a new vm here but im unsure if that's a wise choice.
            // If the issue is recurring this will continue to break workers in a loop effectively
            // will have to do some thinking on this one.
        }
    }

    fn cancel_old_vm_session_pending_acks(&mut self, session_id: u64) {
        let to_cancel = self
            .pending_acks
            .iter()
            .filter_map(|(id, pending)| {
                (pending.dispatched_session_id <= session_id).then_some(*id)
            })
            .collect::<Vec<_>>();

        for id in to_cancel {
            self.cancel_pending_ack(id)
        }
    }

    fn cancel_pending_ack(&mut self, evt_id: u64) {
        let Some(pending) = self.pending_acks.remove(&evt_id) else {
            return;
        };

        match pending.kind {
            PendingAckType::Dispatch(_) => {}
            PendingAckType::ScheduledTask(task) => {
                self.scheduled_tasks_man.remove_pending(task);
            }
            PendingAckType::IntervalTimer(timer) => {
                self.interval_timers_man.remove_pending(timer);
            }
            PendingAckType::Restart => {}
        }
    }

    fn clear_loaded_timers_and_tasks(&mut self) {
        self.interval_timers_man.clear_loaded_timers();
        self.scheduled_tasks_man.clear_task_names();
        self.scheduled_tasks_man.clear_next();
    }

    fn clear_all_pending_timer_acks(&mut self) {
        self.interval_timers_man.clear_pending_acks();
        self.scheduled_tasks_man.clear_pending();
    }

    fn should_send_scripts(&mut self, wr: WorkerRetrieved) -> bool {
        if !self.config.no_reuse_vms
            && !self.force_load_scripts_next
            && matches!(wr, WorkerRetrieved::SameGuild)
        {
            return false;
        }

        true
    }

    fn gen_dispatch_id(&mut self) -> u64 {
        self.dispatch_id_gen += 1;
        self.dispatch_id_gen
    }

    fn invalidate_create_new_session_id(&mut self) -> u64 {
        self.current_vm_session_id += 1;
        self.current_vm_session_id
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
            error!(%err, script_id = evt.script_id.0, "failed updating db contribs",);
        }

        if let Err(err) = self
            .stores
            .update_script(
                self.guild_id,
                UpdateScript {
                    id: evt.script_id.0,
                    name: None,
                    original_source: None,
                    enabled: None,
                    contributes: None,
                    plugin_version_number: None,
                    settings_definitions: Some(evt.settings.clone()),
                    settings_values: None,
                },
            )
            .await
        {
            error!(%err, script_id = evt.script_id.0, "failed updating db contribs (settings)");
        }
    }
}

pub enum NextAction {
    WorkerMessage(Option<WorkerMessage>),
    CheckScheduledTasks,
    CheckIntervalTimers,
}

pub enum VmSessionEvent {
    ShutdownTooManyInvalidRequests,
    ShutdownExcessCpu,
}

pub struct PendingAck {
    dispatched_session_id: u64,
    kind: PendingAckType,
}

pub enum PendingAckType {
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

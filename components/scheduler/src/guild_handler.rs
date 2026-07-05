use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    command_manager,
    interval_timer_manager::{self, TimerId},
    scheduled_task_manager,
    vm_session::{PendingAckType, SessionEvent, VmSession, VmSessionEvent},
    SchedulerConfig,
};
use chrono::{DateTime, Utc};
use common::dispatch_event::EventSource;
use dbrokerapi::broker_scheduler_rpc::{DiscordEvent, DiscordEventData};
use guild_logger::{GuildLogSender, LogSender};
use runtime_models::{internal::script::ScriptMeta, util::PluginId};
use stores::{
    config::{
        hash_script_source, hash_settings_values, IntervalTimerContrib, PremiumSlotTier, Script,
        ScriptContributes, ScriptDerivedFreshness, UpdateScriptDerived, SCRIPT_RUNTIME_VERSION,
    },
    timers::{IntervalTimer, ScheduledTask},
    Db,
};
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, instrument, warn};
use twilight_model::id::{marker::GuildMarker, Id};
use vm::vm::ShutdownReason;

pub enum GuildCommand {
    BrokerEvent(DiscordEvent),
    Status(oneshot::Sender<Option<GuildStatus>>),
    ReloadScripts,
    PurgeCache,
    Shutdown,
}

#[derive(Clone, Copy)]
pub enum PremiumTierState {
    Fetched(Option<PremiumSlotTier>),
    Unknown,
}

impl PremiumTierState {
    pub fn option(&self) -> Option<PremiumSlotTier> {
        match self {
            PremiumTierState::Fetched(inner) => *inner,
            PremiumTierState::Unknown => None,
        }
    }
}

pub struct GuildHandler {
    guild_id: Id<GuildMarker>,

    config: Arc<SchedulerConfig>,
    stores: Db,
    logger: GuildLogSender,
    worker_pool: crate::vmworkerpool::VmWorkerPool,
    scheduler_tx: mpsc::UnboundedSender<VmSessionEvent>,
    guild_rx: mpsc::UnboundedReceiver<GuildCommand>,
    cmd_manager_handle: command_manager::Handle,

    premium_tier: PremiumTierState,

    scripts: Vec<Script>,
    // inputs the stored derived state of each script was observed under,
    // used to skip the db write in script_loaded when nothing changed
    derived_freshness: HashMap<u64, ScriptDerivedFreshness>,
    force_load_scripts_next: bool,

    interval_timers_man: interval_timer_manager::Manager,
    scheduled_tasks_man: scheduled_task_manager::Manager,

    /// Present while we hold a claimed worker; the vm keeps running inside the
    /// worker between claims and is resumed from the session state on the
    /// handle when possible.
    session: Option<VmSession>,
    vm_session_id_gen: u64,

    last_claimed_worker_id: Option<u64>,
    last_claimed_worker_at: Instant,
    last_returned_worker_at: Instant,
}

impl GuildHandler {
    pub fn new_run(
        config: Arc<SchedulerConfig>,
        stores: Db,
        guild_id: Id<GuildMarker>,
        logger: LogSender,
        worker_pool: crate::vmworkerpool::VmWorkerPool,
        cmd_manager_handle: crate::command_manager::Handle,
    ) -> GuildHandle {
        let (handler, handle) = Self::new(
            config,
            stores,
            guild_id,
            logger,
            worker_pool,
            cmd_manager_handle,
        );

        tokio::spawn(handler.run());

        handle
    }

    pub(crate) fn new(
        config: Arc<SchedulerConfig>,
        stores: Db,
        guild_id: Id<GuildMarker>,
        logger: LogSender,
        worker_pool: crate::vmworkerpool::VmWorkerPool,
        cmd_manager_handle: crate::command_manager::Handle,
    ) -> (GuildHandler, GuildHandle) {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let (evt_tx, evt_rx) = mpsc::unbounded_channel();
        let handle = GuildHandle {
            guild_id,

            rx: evt_rx,
            tx: Some(cmd_tx),
        };

        let handler = GuildHandler {
            guild_id,
            stores: stores.clone(),
            logger: logger.with_guild(guild_id),
            worker_pool,

            guild_rx: cmd_rx,
            scheduler_tx: evt_tx,
            cmd_manager_handle,
            config,

            premium_tier: PremiumTierState::Unknown,

            scripts: Vec::new(),
            derived_freshness: HashMap::new(),
            force_load_scripts_next: false,

            interval_timers_man: interval_timer_manager::Manager::new(guild_id, stores.clone()),
            scheduled_tasks_man: scheduled_task_manager::Manager::new(guild_id, stores),

            session: None,
            vm_session_id_gen: 0,

            last_claimed_worker_id: None,
            last_claimed_worker_at: Instant::now(),
            last_returned_worker_at: Instant::now(),
        };

        (handler, handle)
    }

    #[instrument(skip(self), fields(guild_id = self.guild_id.get()))]
    async fn setup(&mut self) {
        self.fetch_premium_tier().await;
        self.try_retry_load_guild_scripts().await;
        self.start_fresh_vm().await;
    }

    async fn run(mut self) {
        self.setup().await;

        while let Some(next) = self.next_event().await {
            if !self.handle_next_action(next).await {
                break;
            }
        }

        self.shutdown().await;
    }

    #[instrument(skip(self), fields(guild_id = self.guild_id.get(), action = action.span_info()))]
    pub(crate) async fn handle_next_action(&mut self, action: NextGuildAction) -> bool {
        match action {
            NextGuildAction::GuildCommand(GuildCommand::Shutdown) => {
                info!("got shutdown signal");
                return false;
            }
            NextGuildAction::GuildCommand(cmd) => {
                self.handle_guild_command(cmd).await;
            }
            NextGuildAction::WorkerMessage(msg) => {
                let Some(session) = &mut self.session else {
                    return true;
                };

                let events = session.handle_worker_msg(msg);
                return self.handle_session_events(events).await;
            }
            NextGuildAction::CheckScheduledTasks => {
                let tasks = self.scheduled_tasks_man.start_triggered_tasks().await;
                for task in tasks {
                    self.dispatch_scheduled_task(task).await;
                }
            }
            NextGuildAction::CheckIntervalTimers => {
                let timers = self.interval_timers_man.trigger_timers().await;
                for timer in timers {
                    self.dispatch_interval_timer(timer).await;
                }
            }
        }

        true
    }

    pub(crate) async fn next_event(&mut self) -> Option<NextGuildAction> {
        self.scheduled_tasks_man.init_next_task_time().await;

        let scheduled_task_sleep_check = match self.scheduled_tasks_man.next_action() {
            scheduled_task_manager::NextAction::None => tokio::time::sleep(Duration::MAX),
            scheduled_task_manager::NextAction::Wait(until) => {
                let sleep_dur = (until - chrono::Utc::now())
                    .to_std()
                    .unwrap_or_else(|_| Duration::from_millis(1));
                tokio::time::sleep(sleep_dur)
            }
            scheduled_task_manager::NextAction::Run => {
                return Some(NextGuildAction::CheckScheduledTasks);
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
                return Some(NextGuildAction::CheckIntervalTimers);
            }
        };

        tokio::pin!(scheduled_task_sleep_check);
        tokio::pin!(interval_timers_sleep_check);

        if let Some(session) = &mut self.session {
            tokio::select! {
                msg = session.recv() => {
                    Some(NextGuildAction::WorkerMessage(msg))
                },
                next_guild_evt = self.guild_rx.recv() => {
                    next_guild_evt.map(NextGuildAction::GuildCommand)
                },
                _ = scheduled_task_sleep_check => {
                    Some(NextGuildAction::CheckScheduledTasks)
                },
                _ = interval_timers_sleep_check => {
                    Some(NextGuildAction::CheckIntervalTimers)
                }
            }
        } else {
            tokio::select! {
                next_guild_evt = self.guild_rx.recv() => {
                    next_guild_evt.map(NextGuildAction::GuildCommand)
                },
                _ = scheduled_task_sleep_check => {
                    Some(NextGuildAction::CheckScheduledTasks)
                },
                _ = interval_timers_sleep_check => {
                    Some(NextGuildAction::CheckIntervalTimers)
                }
            }
        }
    }

    async fn handle_guild_command(&mut self, cmd: GuildCommand) {
        match cmd {
            GuildCommand::BrokerEvent(evt) => {
                self.handle_broker_event(evt).await;
            }
            GuildCommand::ReloadScripts => {
                self.reload_guild_scripts().await;
            }
            GuildCommand::Shutdown => {
                panic!("shutdown should be handled by caller")
            }
            GuildCommand::PurgeCache => {}
            GuildCommand::Status(resp) => {
                let _ = resp.send(Some(self.get_status()));
            }
        }
    }

    fn get_status(&self) -> GuildStatus {
        GuildStatus {
            vm: VmSessionStatus {
                current_claimed_worker: self.session.as_ref().map(|v| v.worker_id()),
                claimed_worker_at: self.last_claimed_worker_at,
                returned_worker_at: self.last_returned_worker_at,
                last_claimed_worker: self.last_claimed_worker_id,
                num_pending_acks: self
                    .session
                    .as_ref()
                    .map(|v| v.num_pending_acks())
                    .unwrap_or(0),
            },
        }
    }

    async fn handle_broker_event(&mut self, evt: DiscordEvent) {
        crate::dispatch_metrics::record_stage("guild_handler_recv", "discord", evt.timestamp);

        match &evt.event {
            DiscordEventData::GuildCreate(_) => {}
            DiscordEventData::GuildDelete(_) => {
                unreachable!("this event should not be forwarded to the guild worker");
            }
            _ => {
                self.send_discord_guild_event(evt).await;
            }
        }
    }

    /// Applies the side effects of the events produced by the session,
    /// returning false when the guild handler should stop.
    async fn handle_session_events(&mut self, events: Vec<SessionEvent>) -> bool {
        let mut keep_running = true;
        let mut broken_session_cleanup = false;

        let mut queue = VecDeque::from(events);
        while let Some(evt) = queue.pop_front() {
            match evt {
                SessionEvent::ScriptStarted(meta) => {
                    self.script_loaded(meta).await;
                }
                SessionEvent::TaskAcked(task_id) => {
                    self.scheduled_tasks_man.ack_triggered_task(task_id).await;
                }
                SessionEvent::TimerAcked(timer) => {
                    self.interval_timers_man.timer_ack(&timer).await;
                }
                SessionEvent::TaskDispatchCancelled(task_id) => {
                    self.scheduled_tasks_man.remove_pending(task_id);
                }
                SessionEvent::TimerDispatchCancelled(timer) => {
                    self.interval_timers_man.remove_pending(timer);
                }
                SessionEvent::TaskScheduled => {
                    self.scheduled_tasks_man.clear_next();
                }
                SessionEvent::VmIdle => {
                    if let Some(session) = self.session.take() {
                        self.note_worker_returned(session.worker_id());
                        session.return_idle(&self.worker_pool);
                    }
                }
                SessionEvent::VmShutdown { current_vm, reason } => {
                    if current_vm {
                        if let Some(session) = self.session.take() {
                            self.note_worker_returned(session.worker_id());
                            session.return_after_shutdown(&self.worker_pool);
                        }
                        self.force_load_scripts_next = true;
                    } else {
                        // an older vm of ours finished shutting down, the new
                        // vm will send us a new set of timers and tasks
                        self.scheduled_tasks_man.clear_task_names();
                        self.interval_timers_man.clear_loaded_timers();
                    }

                    match reason {
                        Some(ShutdownReason::DiscordInvalidRequestsRatelimit) => {
                            let _ = self
                                .scheduler_tx
                                .send(VmSessionEvent::ShutdownTooManyInvalidRequests);
                            keep_running = false;
                        }
                        Some(ShutdownReason::Runaway) => {
                            let _ = self.scheduler_tx.send(VmSessionEvent::ShutdownExcessCpu);
                            keep_running = false;
                        }
                        _ => {}
                    }
                }
                SessionEvent::Broken => {
                    if let Some(session) = self.session.take() {
                        self.note_worker_returned(session.worker_id());
                        queue.extend(session.destroy_broken(&self.worker_pool));
                    }
                    broken_session_cleanup = true;
                }
            }
        }

        if broken_session_cleanup {
            // pending dispatches died with the session
            self.interval_timers_man.clear_pending_acks();
            self.scheduled_tasks_man.clear_pending();
            self.clear_loaded_timers_and_tasks();

            // We could start a new vm here but im unsure if that's a wise choice.
            // If the issue is recurring this will continue to break workers in a loop effectively
            // will have to do some thinking on this one.
        }

        keep_running
    }

    /// Brings up a vm for the current script set, restarting the vm on the
    /// active session if there is one.
    async fn start_fresh_vm(&mut self) {
        info!("loading contribs");

        if self.scripts.is_empty() {
            self.cmd_manager_handle
                .send_no_scripts_enabled(self.guild_id);
        }

        if self.session.is_some() {
            self.vm_session_id_gen += 1;
            let new_session_id = self.vm_session_id_gen;
            let scripts = self.scripts.clone();
            let premium_tier = self.premium_tier.option();

            let session = self.session.as_mut().unwrap();
            if session
                .restart_vm(new_session_id, scripts, premium_tier)
                .is_err()
            {
                self.discard_broken_session().await;
            }
        } else {
            // if we have not claimed a worker then there's no pending acks
            self.clear_loaded_timers_and_tasks();

            if self.scripts.is_empty() {
                return;
            }

            self.force_load_scripts_next = true;
            self.ensure_session().await;
        }
    }

    /// Claims a worker and resumes or creates a vm on it, retrying with new
    /// workers until one accepts.
    #[instrument(skip_all)]
    async fn ensure_session(&mut self) {
        if self.session.is_some() {
            return;
        }

        loop {
            let worker = self
                .worker_pool
                .req_worker(self.guild_id, self.premium_tier.option())
                .await;

            info!(tier = worker.priority_index, "claimed new worker");
            self.last_claimed_worker_at = Instant::now();

            let can_resume = !self.config.no_reuse_vms
                && !self.force_load_scripts_next
                && matches!(&worker.session_state, Some(s) if s.guild_id == self.guild_id);

            if can_resume {
                self.session = Some(VmSession::resume(
                    worker,
                    self.guild_id,
                    self.logger.clone(),
                ));
            } else {
                // a new vm will send us a new set of timers and tasks
                self.clear_loaded_timers_and_tasks();

                self.vm_session_id_gen += 1;
                match VmSession::create(
                    worker,
                    self.guild_id,
                    self.logger.clone(),
                    self.vm_session_id_gen,
                    self.scripts.clone(),
                    self.premium_tier.option(),
                ) {
                    Ok(session) => self.session = Some(session),
                    Err(worker) => {
                        self.note_worker_returned(worker.worker_id);
                        self.worker_pool.return_worker(worker, true);
                        continue;
                    }
                }
            }

            self.force_load_scripts_next = false;
            return;
        }
    }

    async fn discard_broken_session(&mut self) {
        // reason-based scheduler notifications are still sent from within,
        // only the stop signal is dropped here
        let _ = self.handle_session_events(vec![SessionEvent::Broken]).await;
    }

    pub(crate) async fn reload_guild_scripts(&mut self) {
        self.try_retry_load_guild_scripts().await;
        self.start_fresh_vm().await;
    }

    async fn try_retry_load_guild_scripts(&mut self) {
        loop {
            let scripts = match self.stores.list_scripts(self.guild_id).await {
                Ok(scripts) => scripts,
                Err(err) => {
                    error!(%err, "failed loading guild scripts, retrying in 10 secs");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    continue;
                }
            };

            let freshness = match self.stores.get_script_derived_freshness(self.guild_id).await {
                Ok(freshness) => freshness,
                Err(err) => {
                    error!(%err, "failed loading derived script state, retrying in 10 secs");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    continue;
                }
            };

            self.scripts = scripts.into_iter().filter(|v| v.enabled).collect();
            self.derived_freshness = freshness.into_iter().map(|v| (v.script_id, v)).collect();
            return;
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
        self.inner_send_discord_guild_event(evt, None).await;
    }

    /// Same as [Self::send_discord_guild_event] but returns a receiver that resolves
    /// when the event has been acked by the vm.
    ///
    /// Returns None if the event was not dispatched (no scripts, unknown event
    /// type or conversion failure). Currently only used for benchmarking.
    pub(crate) async fn send_discord_guild_event_tracked(
        &mut self,
        evt: DiscordEvent,
    ) -> Option<oneshot::Receiver<()>> {
        let (tx, rx) = oneshot::channel();
        if self.inner_send_discord_guild_event(evt, Some(tx)).await {
            Some(rx)
        } else {
            None
        }
    }

    async fn inner_send_discord_guild_event(
        &mut self,
        evt: DiscordEvent,
        resp: Option<oneshot::Sender<()>>,
    ) -> bool {
        let t_clone = evt.t.clone();
        let ts_clone = evt.timestamp;
        match crate::dispatch_conv::discord_event_to_dispatch(evt) {
            Ok(Some(converted_evt)) => {
                if self.scripts.is_empty() {
                    return false;
                }

                self.dispatch_worker_evt(
                    converted_evt.name.to_string(),
                    converted_evt.data,
                    PendingAckType::Dispatch(resp),
                    EventSource::Discord,
                    ts_clone,
                )
                .await;
                true
            }
            Ok(None) => {
                tracing::warn!(t = t_clone, "skipped converting dispatch event");
                false
            }
            Err(err) => {
                error!(%err, t=t_clone, "failed converting dispatch event");
                false
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

        let mut ack = ack;
        loop {
            self.ensure_session().await;

            let session = self.session.as_mut().expect("ensure_session claims a worker");
            match session.dispatch(t.clone(), data.clone(), ack, source, ts) {
                Ok(()) => {
                    crate::dispatch_metrics::record_stage(
                        "worker_sent",
                        crate::dispatch_metrics::event_source_label(source),
                        ts,
                    );
                    return;
                }
                Err(returned_ack) => {
                    ack = returned_ack;
                    error!("worker died while trying to dispatch event, retrying in a second");
                    self.discard_broken_session().await;
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    #[instrument(skip(self), fields(guild_id = self.guild_id.get()))]
    pub(crate) async fn shutdown(&mut self) {
        info!("shutting down guild handler");

        {
            let Some(session) = &mut self.session else {
                return;
            };

            // wait until the vm has finished its work
            if session.send_complete().is_err() {
                self.discard_broken_session().await;
                return;
            }
        }

        while let Some(session) = &mut self.session {
            let msg = session.recv().await;
            let events = session
                .handle_worker_msg(msg)
                .into_iter()
                // we asked the vm to shut down, keep the worker claimed until
                // the shutdown message arrives instead of returning it on idle
                .filter(|evt| !matches!(evt, SessionEvent::VmIdle))
                .collect();

            self.handle_session_events(events).await;
        }
    }

    async fn fetch_premium_tier(&mut self) {
        self.premium_tier = PremiumTierState::Unknown;

        let slots = if let Ok(slots) = self.stores.get_guild_premium_slots(self.guild_id).await {
            slots
        } else {
            return;
        };

        let mut highest_tier = Option::<PremiumSlotTier>::None;
        for slot in slots {
            if let Some(current_highest) = highest_tier {
                if slot.tier.is_higher_than(current_highest) {
                    highest_tier = Some(slot.tier);
                }
            } else {
                highest_tier = Some(slot.tier);
            }
        }

        self.premium_tier = PremiumTierState::Fetched(highest_tier);
    }

    pub(crate) fn set_premium_tier(&mut self, tier: PremiumTierState) {
        self.premium_tier = tier;
    }

    fn note_worker_returned(&mut self, worker_id: u64) {
        self.last_claimed_worker_id = Some(worker_id);
        self.last_returned_worker_at = Instant::now();
    }

    fn clear_loaded_timers_and_tasks(&mut self) {
        self.interval_timers_man.clear_loaded_timers();
        self.scheduled_tasks_man.clear_task_names();
        self.scheduled_tasks_man.clear_next();
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

        self.update_db_derived_state(&evt, interval_contribs.clone())
            .await;

        self.interval_timers_man
            .script_started(interval_contribs)
            .await;

        self.scheduled_tasks_man.script_started(&evt);

        self.cmd_manager_handle
            .send_loaded_script(self.guild_id, evt);
    }

    async fn update_db_derived_state(
        &mut self,
        evt: &ScriptMeta,
        interval_contribs: Vec<IntervalTimerContrib>,
    ) {
        let script_id = evt.script_id.0;
        let Some(script) = self.scripts.iter().find(|v| v.id == script_id) else {
            warn!(script_id, "loaded script is not in the session script list");
            return;
        };

        let source_hash = hash_script_source(&script.original_source);
        let settings_hash = hash_settings_values(&script.settings_values);

        if let Some(stored) = self.derived_freshness.get(&script_id) {
            if stored.is_fresh(&source_hash, &settings_hash) {
                return;
            }
        }

        let twilight_commands = crate::command_manager::to_twilight_commands(
            self.guild_id,
            &evt.commands,
            &evt.command_groups,
        );

        // TODO: handle errors here, maybe retry?
        if let Err(err) = self
            .stores
            .upsert_script_derived(
                self.guild_id,
                script_id,
                UpdateScriptDerived {
                    source_hash: source_hash.clone(),
                    settings_hash: settings_hash.clone(),
                    contributes: ScriptContributes {
                        commands: twilight_commands,
                        interval_timers: interval_contribs,
                    },
                    settings_definitions: evt.settings.clone(),
                },
            )
            .await
        {
            error!(%err, script_id, "failed updating derived script state");
            return;
        }

        self.derived_freshness.insert(
            script_id,
            ScriptDerivedFreshness {
                script_id,
                source_hash,
                settings_hash,
                runtime_version: SCRIPT_RUNTIME_VERSION.to_string(),
            },
        );
    }
}

pub(crate) enum NextGuildAction {
    WorkerMessage(Option<scheduler_worker_rpc::WorkerMessage>),
    GuildCommand(GuildCommand),
    CheckScheduledTasks,
    CheckIntervalTimers,
}

impl NextGuildAction {
    fn span_info(&self) -> String {
        match self {
            NextGuildAction::WorkerMessage(wm) => {
                let wm_name = wm.as_ref().map(|v| v.name()).unwrap_or("none");
                format!("WorkerMessage({})", wm_name)
            }
            NextGuildAction::CheckScheduledTasks => "CheckScheduledTasks".to_owned(),
            NextGuildAction::CheckIntervalTimers => "CheckIntervalTimers".to_owned(),
            NextGuildAction::GuildCommand(cmd) => match cmd {
                GuildCommand::BrokerEvent(be) => format!("GuildCommand(BrokerEvent({}))", be.t),
                GuildCommand::ReloadScripts => "GuildCommand(ReloadScripts)".to_owned(),
                GuildCommand::PurgeCache => "GuildCommand(PurgeCache)".to_owned(),
                GuildCommand::Shutdown => "GuildCommand(Shutdown)".to_owned(),
                GuildCommand::Status(_) => "GuildCommand(Status)".to_owned(),
            },
        }
    }
}

pub struct GuildHandle {
    pub guild_id: Id<GuildMarker>,
    pub rx: mpsc::UnboundedReceiver<VmSessionEvent>,
    pub tx: Option<mpsc::UnboundedSender<GuildCommand>>,
}

pub enum NextTimerAction {
    None,
    Wait(DateTime<Utc>),
    Run,
}

pub struct GuildStatus {
    pub vm: VmSessionStatus,
}

pub struct VmSessionStatus {
    pub current_claimed_worker: Option<u64>,
    pub last_claimed_worker: Option<u64>,
    pub claimed_worker_at: Instant,
    pub returned_worker_at: Instant,
    pub num_pending_acks: usize,
}

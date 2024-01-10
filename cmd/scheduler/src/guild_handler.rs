use std::sync::{Arc, RwLock};

use crate::{
    command_manager,
    scheduler::Store,
    vm_session::{VmSession, VmSessionEvent},
};
use chrono::{DateTime, Utc};
use common::DiscordConfig;
use dbrokerapi::broker_scheduler_rpc::GuildEvent;
use guild_logger::LogSender;
use stores::config::PremiumSlotTier;
use tokio::sync::mpsc;
use tracing::{info, instrument};
use twilight_model::{
    gateway::event::DispatchEvent,
    id::{marker::GuildMarker, Id},
};

pub enum GuildCommand {
    BrokerEvent(GuildEvent),
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

    _discord_config: Arc<DiscordConfig>,
    stores: Arc<dyn Store>,
    _logger: LogSender,
    _worker_pool: crate::vmworkerpool::VmWorkerPool,
    scheduler_tx: mpsc::UnboundedSender<VmSessionEvent>,
    guild_rx: mpsc::UnboundedReceiver<GuildCommand>,
    _cmd_manager_handle: command_manager::Handle,

    premium_tier: Arc<RwLock<PremiumTierState>>,

    _id_gen: u64,

    scripts_session: VmSession,
}

impl GuildHandler {
    pub fn new_run(
        stores: Arc<dyn Store>,
        guild_id: Id<GuildMarker>,
        logger: LogSender,
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

        let premium_tier = Arc::new(RwLock::new(PremiumTierState::Unknown));

        let worker = GuildHandler {
            stores: stores.clone(),
            guild_id,
            _logger: logger.clone(),
            _worker_pool: worker_pool.clone(),
            _discord_config: discord_config.clone(),

            guild_rx: cmd_rx,
            scheduler_tx: evt_tx,
            _id_gen: 1,
            premium_tier: premium_tier.clone(),

            _cmd_manager_handle: cmd_manager_handle.clone(),
            scripts_session: VmSession::new(
                stores,
                guild_id,
                logger.with_guild(guild_id),
                worker_pool,
                cmd_manager_handle,
                discord_config,
                premium_tier,
            ),
        };

        tokio::spawn(worker.run());

        handle
    }

    #[instrument(skip(self), fields(guild_id = self.guild_id.get()))]
    async fn setup(&mut self) {
        self.fetch_premium_tier().await;
        self.scripts_session.start().await;
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
    async fn handle_next_action(&mut self, action: NextGuildAction) -> bool {
        match action {
            NextGuildAction::GuildCommand(GuildCommand::Shutdown) => {
                info!("got shutdown signal");
                return false;
            }
            NextGuildAction::GuildCommand(cmd) => {
                self.handle_guild_command(cmd).await;
            }
            NextGuildAction::VmAction(action) => {
                if let Some(evt) = self.scripts_session.handle_action(action).await {
                    match evt {
                        crate::vm_session::VmSessionEvent::TooManyInvalidRequests => {
                            let _ = self.scheduler_tx.send(evt);
                            return false;
                        }
                        crate::vm_session::VmSessionEvent::ForciblyShutdown => {
                            let _ = self.scheduler_tx.send(evt);
                            return false;
                        }
                    }
                }
            }
        }

        return true;
    }

    #[instrument(skip(self), fields(guild_id = self.guild_id.get()))]
    async fn shutdown(&mut self) {
        info!("shutting down guild handler");
        self.scripts_session.shutdown().await;
    }

    async fn next_event(&mut self) -> Option<NextGuildAction> {
        self.scripts_session.init_timers().await;

        tokio::select! {
            next_scripts_evt = self.scripts_session.next_action() => {
                Some(NextGuildAction::VmAction(next_scripts_evt))
            },
            next_guild_evt = self.guild_rx.recv() => {
                next_guild_evt.map(NextGuildAction::GuildCommand)
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
                self.scripts_session.reload_guild_scripts().await;
            }
            GuildCommand::Shutdown => {
                panic!("shutdown should be handled by caller")
            }
            GuildCommand::PurgeCache => {}
        }
    }

    async fn fetch_premium_tier(&mut self) {
        {
            let mut w = self.premium_tier.write().unwrap();
            *w = PremiumTierState::Unknown;
        }

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

        let mut w = self.premium_tier.write().unwrap();
        *w = PremiumTierState::Fetched(highest_tier);
    }

    async fn handle_broker_event(&mut self, evt: GuildEvent) {
        match &*evt.event {
            DispatchEvent::GuildCreate(_) => {}
            DispatchEvent::GuildDelete(_) => {
                unreachable!("this event should not be forwarded to the guild worker");
            }
            _ => {
                self.scripts_session.send_discord_guild_event(evt).await;
            }
        }
    }
}

enum NextGuildAction {
    VmAction(crate::vm_session::NextAction),
    GuildCommand(GuildCommand),
}

impl NextGuildAction {
    fn span_info(&self) -> String {
        match self {
            NextGuildAction::VmAction(action) => match action {
                crate::vm_session::NextAction::WorkerMessage(wm) => {
                    let wm_name = wm.as_ref().map(|v| v.name()).unwrap_or("none");
                    format!("VmAction(WorkerMessage({}))", wm_name)
                }
                crate::vm_session::NextAction::CheckScheduledTasks => {
                    "VmAction(CheckScheduledTasks)".to_owned()
                }
                crate::vm_session::NextAction::CheckIntervalTimers => {
                    "VmAction(CheckIntervalTimers)".to_owned()
                }
            },
            NextGuildAction::GuildCommand(cmd) => match cmd {
                GuildCommand::BrokerEvent(be) => format!("GuildCommand(BrokerEvent({}))", be.t),
                GuildCommand::ReloadScripts => "GuildCommand(ReloadScripts)".to_owned(),
                GuildCommand::PurgeCache => "GuildCommand(PurgeCache)".to_owned(),
                GuildCommand::Shutdown => "GuildCommand(Shutdown)".to_owned(),
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

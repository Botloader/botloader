use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use crate::{
    command_manager,
    scheduler::Store,
    vm_session::{CreateSessionType, VmSession, VmSessionEvent},
};
use chrono::{DateTime, Utc};
use common::DiscordConfig;
use dbrokerapi::broker_scheduler_rpc::GuildEvent;
use guild_logger::GuildLogger;
use stores::config::PremiumSlotTier;
use tokio::sync::mpsc;
use tracing::{error, info, instrument};
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

    discord_config: Arc<DiscordConfig>,
    stores: Arc<dyn Store>,
    logger: GuildLogger,
    worker_pool: crate::vmworkerpool::VmWorkerPool,
    scheduler_tx: mpsc::UnboundedSender<VmSessionEvent>,
    guild_rx: mpsc::UnboundedReceiver<GuildCommand>,
    cmd_manager_handle: command_manager::Handle,

    premium_tier: Arc<RwLock<PremiumTierState>>,

    id_gen: u64,

    scripts_session: Option<VmSession>,
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

        let premium_tier = Arc::new(RwLock::new(PremiumTierState::Unknown));

        let worker = GuildHandler {
            stores: stores.clone(),
            guild_id,
            logger: logger.clone(),
            worker_pool: worker_pool.clone(),
            discord_config: discord_config.clone(),

            guild_rx: cmd_rx,
            scheduler_tx: evt_tx,
            id_gen: 1,
            premium_tier: premium_tier.clone(),

            cmd_manager_handle: cmd_manager_handle.clone(),
            scripts_session: None,
        };

        tokio::spawn(worker.run());

        handle
    }

    async fn ensure_scripts_session(&mut self) {
        let scripts_session = match VmSession::new(
            CreateSessionType::GuildScripts,
            self.stores.clone(),
            self.guild_id,
            self.logger.clone(),
            self.worker_pool.clone(),
            self.cmd_manager_handle.clone(),
            self.discord_config.clone(),
            self.premium_tier.clone(),
        )
        .await
        {
            Ok(mut v) => {
                v.start().await;
                Some(v)
            }
            Err(err) => {
                error!(%err, "failed creating guild scripts session");
                None
            }
        };

        self.scripts_session = scripts_session;
    }

    #[instrument(skip(self), fields(guild_id = self.guild_id.get()))]
    async fn run(mut self) {
        self.fetch_premium_tier().await;
        self.ensure_scripts_session().await;

        while let Some(next) = self.next_event().await {
            match next {
                NextGuildAction::GuildCommand(GuildCommand::Shutdown) => {
                    info!("got shutdown signal");
                    break;
                }
                NextGuildAction::GuildCommand(cmd) => {
                    self.handle_guild_command(cmd).await;
                }
                NextGuildAction::VmAction(action) => {
                    if let Some(session) = &mut self.scripts_session {
                        if let Some(evt) = session.handle_action(action).await {
                            match evt {
                                crate::vm_session::VmSessionEvent::TooManyInvalidRequests => {
                                    let _ = self.scheduler_tx.send(evt);
                                    break;
                                }
                                crate::vm_session::VmSessionEvent::ForciblyShutdown => {
                                    let _ = self.scheduler_tx.send(evt);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        info!("shutting down guild handler");
        if let Some(session) = &mut self.scripts_session {
            session.shutdown().await;
        }
    }

    async fn next_event(&mut self) -> Option<NextGuildAction> {
        if let Some(session) = &mut self.scripts_session {
            session.init_timers().await;
            tokio::select! {
                next_scripts_evt = session.next_action() => {
                    Some(NextGuildAction::VmAction(next_scripts_evt))
                },
                next_guild_evt = self.guild_rx.recv() => {
                    next_guild_evt.map(NextGuildAction::GuildCommand)
                }
            }
        } else {
            self.guild_rx
                .recv()
                .await
                .map(NextGuildAction::GuildCommand)
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
                if let Some(session) = &mut self.scripts_session {
                    session.reload_scripts().await;
                }
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
                if let Some(session) = &mut self.scripts_session {
                    session.send_discord_guild_event(evt).await;
                }
            }
        }
    }
}

enum NextGuildAction {
    VmAction(crate::vm_session::NextAction),
    GuildCommand(GuildCommand),
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

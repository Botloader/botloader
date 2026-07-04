//! Helpers for benchmarking the full scheduler side event dispatch flow:
//!
//! event in -> guild goes active (premium tier + scripts loaded from db)
//! -> worker claimed from the pool -> create vm rpc -> vm created and
//! scripts ran -> event dispatched -> acked by the vm.
//!
//! This drives the same code path as [crate::guild_handler::GuildHandler] /
//! [crate::vm_session::VmSession] with real vm worker child processes, it's
//! used by the vmbench binary and is not part of the normal scheduler runtime.

use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use common::DiscordConfig;
use dbrokerapi::broker_scheduler_rpc::{DiscordEvent, DiscordEventData};
use guild_logger::LogSender;
use stores::{
    config::{CreateScript, PremiumSlotTier},
    Db,
};
use twilight_model::{
    gateway::payload::incoming::MessageCreate,
    id::{marker::GuildMarker, Id},
};

use crate::{
    guild_handler::PremiumTierState,
    vm_session::VmSession,
    vmworkerpool::{VmWorkerPool, WorkerLaunchConfig},
    SchedulerConfig,
};

pub struct FullFlowBenchOptions {
    pub db: Db,
    pub discord_config: Arc<DiscordConfig>,
    pub logger: LogSender,
    pub launch_config: WorkerLaunchConfig,
    pub num_workers: usize,
    pub guild_id: Id<GuildMarker>,
}

pub struct BenchScript {
    pub name: String,
    pub source: String,
}

pub struct FullFlowTimings {
    /// Guild activation done: premium tier and scripts fetched from the db,
    /// worker claimed from the pool and the create vm request sent.
    pub vm_requested: Duration,
    /// The dispatched event was acked: the vm was created, all scripts ran and
    /// the event was delivered to the vm.
    pub event_acked: Duration,
}

pub struct FullFlowBencher {
    db: Db,
    // a single long lived session is used across iterations, like a guild
    // handler in production: vm session ids must keep incrementing so stale
    // shutdown messages from a previous vm on a reclaimed worker aren't
    // mistaken for the current vm's shutdown
    session: VmSession,
    premium_tier: Arc<RwLock<PremiumTierState>>,
    // kept alive so command manager sends succeed, but deliberately never ran:
    // command contributions queue up here instead of being synced to the
    // discord api
    _cmd_manager: crate::command_manager::Manager,
    guild_id: Id<GuildMarker>,
}

impl FullFlowBencher {
    /// Replaces the bench guild's scripts in the database with the given ones,
    /// spawns vm worker processes and waits for them to connect.
    pub async fn new(
        opts: FullFlowBenchOptions,
        scripts: Vec<BenchScript>,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            !scripts.is_empty(),
            "the full flow bench needs at least one script, events aren't dispatched to guilds without scripts"
        );

        let existing = opts.db.list_scripts(opts.guild_id).await?;
        for script in existing {
            opts.db.del_script(opts.guild_id, script.name).await?;
        }
        for script in scripts {
            opts.db
                .create_script(
                    opts.guild_id,
                    CreateScript {
                        name: script.name,
                        original_source: script.source,
                        enabled: true,
                        plugin_auto_update: None,
                        plugin_id: None,
                        plugin_version_number: None,
                    },
                )
                .await?;
        }

        let worker_pool = VmWorkerPool::new(opts.launch_config);

        #[cfg(target_family = "unix")]
        crate::worker_listener::listen_for_workers(
            "/tmp/botloader_scheduler_workers",
            worker_pool.clone(),
        )
        .await;

        #[cfg(target_family = "windows")]
        crate::worker_listener::listen_for_workers("localhost:7885", worker_pool.clone()).await;

        worker_pool.spawn_workers(None, opts.num_workers);

        // workers do their own startup (discord config fetch, db connect) before
        // connecting back to us
        while worker_pool.worker_statuses().len() < opts.num_workers {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        let (cmd_manager, cmd_handle) = crate::command_manager::create_manager_pair(
            opts.db.clone(),
            opts.discord_config.clone(),
            opts.logger.clone(),
        );

        let scheduler_config = Arc::new(SchedulerConfig {
            broker_rpc_connect_adddr: String::new(),
            integration_tests_guild: None,
            num_workers_free: 0,
            num_workers_lite: 0,
            num_workers_premium: 0,
            no_reuse_vms: false,
        });

        let premium_tier = Arc::new(RwLock::new(PremiumTierState::Unknown));
        let session = VmSession::new(
            scheduler_config,
            opts.db.clone(),
            opts.guild_id,
            opts.logger.with_guild(opts.guild_id),
            worker_pool,
            cmd_handle,
            premium_tier.clone(),
        );

        Ok(Self {
            db: opts.db,
            session,
            premium_tier,
            _cmd_manager: cmd_manager,
            guild_id: opts.guild_id,
        })
    }

    /// Runs the whole flow once, as if an event arrived for an inactive guild:
    /// guild activation (premium tier + scripts from the db, fresh vm
    /// requested), event dispatch and waiting for the ack.
    pub async fn run_iteration(&mut self) -> anyhow::Result<FullFlowTimings> {
        let started = Instant::now();

        // mirrors GuildHandler::setup for a guild going active
        let tier = self.fetch_premium_tier().await;
        {
            let mut w = self.premium_tier.write().unwrap();
            *w = PremiumTierState::Fetched(tier);
        }
        self.session.reload_guild_scripts().await;

        let vm_requested = started.elapsed();

        let Some(mut ack_rx) = self
            .session
            .send_discord_guild_event_tracked(message_create_event(self.guild_id))
            .await
        else {
            anyhow::bail!("the benchmark event was not dispatched");
        };

        let event_acked = loop {
            tokio::select! {
                biased;
                _ = &mut ack_rx => break started.elapsed(),
                action = self.session.next_action() => {
                    if self.session.handle_action(action).await.is_some() {
                        anyhow::bail!("vm session shut down before the event was acked");
                    }
                }
            }
        };

        // tear the vm down between iterations so the next one starts cold
        self.session.shutdown().await;

        Ok(FullFlowTimings {
            vm_requested,
            event_acked,
        })
    }

    async fn fetch_premium_tier(&self) -> Option<PremiumSlotTier> {
        let Ok(slots) = self.db.get_guild_premium_slots(self.guild_id).await else {
            return None;
        };

        let mut highest_tier = Option::<PremiumSlotTier>::None;
        for slot in slots {
            match highest_tier {
                Some(current) if !slot.tier.is_higher_than(current) => {}
                _ => highest_tier = Some(slot.tier),
            }
        }
        highest_tier
    }
}

fn message_create_event(guild_id: Id<GuildMarker>) -> DiscordEvent {
    let message: MessageCreate = serde_json::from_value(serde_json::json!({
        "id": "3",
        "channel_id": "2",
        "guild_id": guild_id.to_string(),
        "author": {
            "id": "4",
            "username": "vmbench",
            "discriminator": "0001",
            "avatar": null,
            "bot": false,
        },
        "member": {
            "roles": [],
            "joined_at": "2020-02-02T02:02:02.020000+00:00",
            "deaf": false,
            "mute": false,
            "flags": 0,
        },
        "content": "hello from vmbench",
        "timestamp": "2020-02-02T02:02:02.020000+00:00",
        "edited_timestamp": null,
        "tts": false,
        "mention_everyone": false,
        "mentions": [],
        "mention_roles": [],
        "attachments": [],
        "embeds": [],
        "pinned": false,
        "type": 0,
    }))
    .expect("valid stub MessageCreate");

    DiscordEvent {
        t: "MESSAGE_CREATE".to_string(),
        guild_id,
        event: DiscordEventData::MessageCreate(Box::new(message)),
        timestamp: chrono::Utc::now(),
    }
}

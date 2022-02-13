use std::{
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

use dbrokerapi::broker_scheduler_rpc::{HelloData, RawDiscordEvent};
use futures_util::StreamExt;

use stores::config::ConfigStore;
use tokio::{
    net::TcpStream,
    sync::mpsc::{self},
};
use tracing::{error, info, warn};
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::{cluster::Events, Cluster, Event, Intents};
use twilight_model::{gateway::event::DispatchEvent, id::GuildId};

pub async fn run_broker(
    token: String,
    discord_state: Arc<InMemoryCache>,
    stores: Arc<dyn ConfigStore>,
    ready: Arc<AtomicBool>,
) -> Result<BrokerHandle, Box<dyn std::error::Error>> {
    let intents = Intents::GUILD_MESSAGES
        | Intents::GUILDS
        | Intents::GUILD_MEMBERS
        | Intents::GUILD_BANS
        | Intents::GUILD_INVITES
        | Intents::GUILD_VOICE_STATES
        | Intents::GUILD_MESSAGES
        | Intents::GUILD_MESSAGE_REACTIONS;

    let (cluster, events) = Cluster::new(token, intents).await?;
    let cluster = Arc::new(cluster);

    let cluster_spawn = cluster.clone();
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

    let mut discord_manager = Broker {
        _cluster: cluster,
        discord_state,
        events,
        cmd_rx,
        stores,
        ready,
        connected_scheduler: None,
        queued_events: Vec::new(),
        scheduler_discconected_at: Instant::now(),
    };

    tokio::spawn(async move { discord_manager.run().await });

    Ok(cmd_tx)
}

pub type BrokerHandle = mpsc::UnboundedSender<BrokerCommand>;

struct Broker {
    _cluster: Arc<Cluster>,
    discord_state: Arc<InMemoryCache>,
    events: Events,
    cmd_rx: mpsc::UnboundedReceiver<BrokerCommand>,

    connected_scheduler: Option<TcpStream>,
    queued_events: Vec<(GuildId, DispatchEvent)>,
    scheduler_discconected_at: Instant,
    stores: Arc<dyn ConfigStore>,
    ready: Arc<AtomicBool>,
    // scheduler_client: Option<BrokerSchedulerServiceClient>,
    // scheduler_addr: String,
}

impl Broker {
    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                evt = self.events.next() => match evt {
                    Some((_shard_id, evt)) => {
                        self.handle_event(evt).await;
                    },
                    None => todo!(),
                },
                cmd = self.cmd_rx.recv() => match cmd{
                    Some(cmd) => self.handle_cmd(cmd).await,
                    None => todo!(),
                },
            }
        }
    }

    async fn handle_cmd(&mut self, cmd: BrokerCommand) {
        match cmd {
            BrokerCommand::SchedulerConnected(stream) => {
                info!("scheduler connected");
                self.connected_scheduler = Some(stream);
                self.handle_new_scheduler_connected().await;
            }
        }
    }

    async fn handle_event(&mut self, evt: Event) {
        self.discord_state.update(&evt);
        metrics::counter!("bl.broker.handled_events_total", 1);

        let guild_id = match &evt {
            Event::Ready(_) => {
                self.ready.store(true, std::sync::atomic::Ordering::SeqCst);

                metrics::gauge!("bl.broker.connected_guilds_total", 0.0);
                info!("received ready!");
                return;
            }
            Event::GuildDelete(g) => {
                metrics::decrement_gauge!("bl.broker.connected_guilds_total", 1.0);

                if !g.unavailable {
                    let _ = self.stores.set_guild_left_status(g.id, true).await;
                }

                g.id
            }
            Event::GuildCreate(gc) => {
                let _ = self
                    .stores
                    .add_update_joined_guild(stores::config::JoinedGuild {
                        id: gc.id,
                        name: gc.name.clone(),
                        icon: gc.icon.clone().unwrap_or_default(),
                        owner_id: gc.owner_id,
                        left_at: None,
                    })
                    .await;

                metrics::increment_gauge!("bl.broker.connected_guilds_total", 1.0);
                gc.id
            }
            Event::MemberAdd(m) => m.guild_id,
            Event::MemberRemove(m) => m.guild_id,
            Event::MemberUpdate(m) => m.guild_id,
            Event::MessageCreate(m) => {
                if let Some(guild_id) = m.guild_id {
                    guild_id
                } else {
                    return;
                }
            }
            Event::MessageDelete(m) => {
                if let Some(guild_id) = m.guild_id {
                    guild_id
                } else {
                    return;
                }
            }
            Event::MessageDeleteBulk(m) => {
                if let Some(guild_id) = m.guild_id {
                    guild_id
                } else {
                    return;
                }
            }
            Event::MessageUpdate(m) => {
                if let Some(guild_id) = m.guild_id {
                    guild_id
                } else {
                    return;
                }
            }

            Event::InteractionCreate(i) => {
                if let Some(guild_id) = i.guild_id() {
                    guild_id
                } else {
                    return;
                }
            }
            _ => return,
        };

        if let Ok(dispatch) = DispatchEvent::try_from(evt) {
            metrics::counter!("bl.broker.dispatched_events", 1);
            self.dispatch_or_queue_event(guild_id, dispatch).await;
        }
    }

    async fn handle_new_scheduler_connected(&mut self) {
        // send connected guilds
        let guilds = self
            .discord_state
            .iter()
            .guilds()
            .map(|v| v.id())
            .collect::<Vec<_>>();
        if self
            .send_event(dbrokerapi::broker_scheduler_rpc::BrokerEvent::Hello(
                HelloData {
                    connected_guilds: guilds,
                },
            ))
            .await
            .is_err()
        {
            self.connected_scheduler = None;
        }

        // send pending events

        let old_queued = std::mem::take(&mut self.queued_events);
        for (guild_id, evt) in old_queued.into_iter() {
            if self.connected_scheduler.is_some() {
                let v = serde_json::to_value(&evt).unwrap();

                if self
                    .send_event(dbrokerapi::broker_scheduler_rpc::BrokerEvent::DiscordEvent(
                        RawDiscordEvent {
                            event: v,
                            guild_id,
                            t: evt.kind().name().unwrap().to_string(),
                        },
                    ))
                    .await
                    .is_err()
                {
                    // connection dead, re-queue
                    self.queued_events.push((guild_id, evt));
                    self.connected_scheduler = None;
                    self.scheduler_discconected_at = Instant::now();
                }
            } else {
                self.queued_events.push((guild_id, evt))
            }
        }
    }

    async fn dispatch_or_queue_event(&mut self, guild_id: GuildId, evt: DispatchEvent) {
        if self.connected_scheduler.is_some() {
            info!("dispatching event");
            let v = serde_json::to_value(&evt).unwrap();
            if self
                .send_event(dbrokerapi::broker_scheduler_rpc::BrokerEvent::DiscordEvent(
                    RawDiscordEvent {
                        event: v,
                        guild_id,
                        t: evt.kind().name().unwrap().to_string(),
                    },
                ))
                .await
                .is_err()
            {
                self.connected_scheduler = None;
                self.scheduler_discconected_at = Instant::now();
                self.queue_event(guild_id, evt);

                error!("Scheduler disconnected, started queueing events");
            }
        } else {
            info!("queued event");
            self.queue_event(guild_id, evt);
        }
    }

    fn queue_event(&mut self, guild_id: GuildId, evt: DispatchEvent) {
        if Instant::elapsed(&self.scheduler_discconected_at) > Duration::from_secs(60) {
            warn!("event queue too old, expired, clearing");
            self.queued_events = Vec::new();
            return;
        }

        self.queued_events.push((guild_id, evt));
    }

    async fn send_event(
        &mut self,
        evt: dbrokerapi::broker_scheduler_rpc::BrokerEvent,
    ) -> std::io::Result<()> {
        if let Some(connected) = &mut self.connected_scheduler {
            simpleproto::write_message(&evt, connected).await?;
            self.wait_for_ack().await?;
        }

        Ok(())
    }

    async fn wait_for_ack(&mut self) -> std::io::Result<()> {
        if let Some(connected) = &mut self.connected_scheduler {
            let _msg: dbrokerapi::broker_scheduler_rpc::SchedulerEvent =
                simpleproto::read_message(connected).await?;
        }

        Ok(())
    }
}

pub enum BrokerCommand {
    SchedulerConnected(TcpStream),
}

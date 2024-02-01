use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

use dbrokerapi::broker_scheduler_rpc::{HelloData, RawDiscordEvent};
use futures_util::StreamExt;

use stores::config::ConfigStore;
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, UnboundedSender},
};
use tracing::{error, info, warn};
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::{
    stream::{self, ShardEventStream},
    Config, Event, Intents, MessageSender, Shard,
};
use twilight_http::Client;
// use twilight_gateway::{cluster::Events, Cluster, stream, Event, Intents};
use twilight_model::{
    gateway::{event::DispatchEvent, payload::outgoing::RequestGuildMembers},
    guild::Member,
    id::{
        marker::{GuildMarker, UserMarker},
        Id,
    },
};

pub async fn run_broker(
    token: String,
    discord_state: Arc<InMemoryCache>,
    stores: Arc<dyn ConfigStore>,
    ready: Arc<AtomicBool>,
) -> Result<BrokerHandle, Box<dyn std::error::Error>> {
    let intents = Intents::GUILD_MESSAGES
        | Intents::MESSAGE_CONTENT
        | Intents::GUILDS
        | Intents::GUILD_MEMBERS
        | Intents::GUILD_MODERATION
        | Intents::GUILD_INVITES
        | Intents::GUILD_VOICE_STATES
        | Intents::GUILD_MESSAGES
        | Intents::GUILD_MESSAGE_REACTIONS;
    let config = Config::new(token.clone(), intents);

    // let (cluster, events) = Cluster::new(token, intents).await?;

    let client = Client::new(token.clone());
    let shards = stream::create_recommended(&client, config, |_, builder| builder.build())
        .await?
        .collect::<Vec<_>>();

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

    let mut discord_manager = Broker {
        discord_state,
        cmd_rx,
        stores,
        ready,
        connected_scheduler: None,
        queued_events: Vec::new(),
        scheduler_discconected_at: Instant::now(),
        gateway_message_senders: shards.iter().map(|v| v.sender()).collect(),
        nonce_counter: 0,
        pending_guild_member_requests: Default::default(),
    };

    tokio::spawn(async move { discord_manager.run(shards).await });

    Ok(cmd_tx)
}

pub type BrokerHandle = mpsc::UnboundedSender<BrokerCommand>;

struct Broker {
    discord_state: Arc<InMemoryCache>,
    cmd_rx: mpsc::UnboundedReceiver<BrokerCommand>,

    connected_scheduler: Option<TcpStream>,
    queued_events: Vec<(Id<GuildMarker>, DispatchEvent)>,
    scheduler_discconected_at: Instant,
    stores: Arc<dyn ConfigStore>,
    ready: Arc<AtomicBool>,
    gateway_message_senders: Vec<MessageSender>,

    nonce_counter: u64,

    // map of pending guild member requests and their nonce
    pending_guild_member_requests: HashMap<String, PendingChunkState>,
}

impl Broker {
    pub async fn run(&mut self, mut shards: Vec<Shard>) {
        let mut stream = ShardEventStream::new(shards.iter_mut());

        // while let Some((shard, event)) = stream.next().await {
        //     let event = match event {
        //         Ok(event) => event,
        //         Err(source) => {
        //             tracing::warn!(?source, "error receiving event");

        //             if source.is_fatal() {
        //                 break;
        //             }

        //             continue;
        //         }
        //     };

        //     tracing::debug!(?event, shard = ?shard.id(), "received event");
        // }

        loop {
            tokio::select! {
                evt = stream.next() => match evt {
                    Some((_shard_id, evt)) => match evt{
                        Ok(evt) => self.handle_event(evt).await,
                        Err(err) => {
                            error!(?err, "failed handling event");
                            if err.is_fatal(){
                                error!(?err, "fatal error occurred");
                                break;
                            }
                        }
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
            BrokerCommand::RequestGuildMembers(req) => {
                self.handle_request_guild_members(req).await;
            }
        }
    }

    async fn handle_event(&mut self, evt: Event) {
        self.discord_state.update(&evt);
        metrics::counter!("bl.broker.handled_events_total").increment(1);

        let forward_for_guild = match &evt {
            Event::Ready(_) => {
                self.ready.store(true, std::sync::atomic::Ordering::SeqCst);

                metrics::gauge!("bl.broker.connected_guilds_total").set(0.0);
                info!("received ready!");
                return;
            }
            Event::GuildDelete(g) => {
                metrics::gauge!("bl.broker.connected_guilds_total").decrement(1.0);

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
                        icon: gc
                            .icon
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_default(),
                        owner_id: gc.owner_id,
                        left_at: None,
                    })
                    .await;

                metrics::gauge!("bl.broker.connected_guilds_total").increment(1.0);
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

            Event::ReactionAdd(r) => {
                if let Some(guild_id) = r.guild_id {
                    guild_id
                } else {
                    return;
                }
            }
            Event::ReactionRemove(r) => {
                if let Some(guild_id) = r.guild_id {
                    guild_id
                } else {
                    return;
                }
            }
            Event::ReactionRemoveAll(r) => {
                if let Some(guild_id) = r.guild_id {
                    guild_id
                } else {
                    return;
                }
            }
            Event::ReactionRemoveEmoji(r) => {
                r.guild_id
                // if let Some(guild_id) = r.guild_id {
                //     guild_id
                // } else {
                //     return;
                // }
            }

            Event::InteractionCreate(i) => {
                if let Some(guild_id) = i.guild_id {
                    guild_id
                } else {
                    return;
                }
            }
            Event::ChannelCreate(v) => {
                if let Some(guild_id) = v.guild_id {
                    guild_id
                } else {
                    return;
                }
            }
            Event::ChannelUpdate(v) => {
                if let Some(guild_id) = v.guild_id {
                    guild_id
                } else {
                    return;
                }
            }
            Event::ChannelDelete(v) => {
                if let Some(guild_id) = v.guild_id {
                    guild_id
                } else {
                    return;
                }
            }
            Event::ThreadCreate(v) => {
                if let Some(guild_id) = v.guild_id {
                    guild_id
                } else {
                    return;
                }
            }
            Event::ThreadUpdate(v) => {
                if let Some(guild_id) = v.guild_id {
                    guild_id
                } else {
                    return;
                }
            }
            Event::ThreadDelete(v) => v.guild_id,
            Event::MemberChunk(chunk) => {
                let nonce = chunk.nonce.clone().unwrap_or_default();
                let Some(state) = self.pending_guild_member_requests.get_mut(&nonce) else {
                    return;
                };

                state.received_chunks += 1;

                let _ = state.response.send(chunk.members.clone());

                if state.received_chunks >= chunk.chunk_count as u64 {
                    self.pending_guild_member_requests.remove(&nonce);
                }

                return;
            }
            Event::InviteCreate(invite) => invite.guild_id,
            Event::InviteDelete(invite) => invite.guild_id,
            _ => return,
        };

        if let Ok(dispatch) = DispatchEvent::try_from(evt) {
            metrics::counter!("bl.broker.dispatched_events").increment(1);
            self.dispatch_or_queue_event(forward_for_guild, dispatch)
                .await;
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

    async fn handle_request_guild_members(&mut self, req: GuildMembersRequest) {
        let destination_shard =
            (req.guild_id.get() >> 22) % self.gateway_message_senders.len() as u64;

        let nonce = self.next_nonce();

        let sender = self
            .gateway_message_senders
            .get(destination_shard as usize)
            .unwrap();

        sender
            .command(
                &RequestGuildMembers::builder(req.guild_id)
                    .nonce(nonce.to_string())
                    .user_ids(req.user_ids)
                    .unwrap(),
            )
            .unwrap();

        self.pending_guild_member_requests.insert(
            nonce.to_string(),
            PendingChunkState {
                response: req.response,
                received_chunks: 0,
            },
        );
    }

    async fn dispatch_or_queue_event(&mut self, guild_id: Id<GuildMarker>, evt: DispatchEvent) {
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

    fn queue_event(&mut self, guild_id: Id<GuildMarker>, evt: DispatchEvent) {
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

    fn next_nonce(&mut self) -> u64 {
        self.nonce_counter += 1;
        self.nonce_counter
    }
}

pub enum BrokerCommand {
    SchedulerConnected(TcpStream),
    RequestGuildMembers(GuildMembersRequest),
}

pub struct GuildMembersRequest {
    pub user_ids: Vec<Id<UserMarker>>,
    pub guild_id: Id<GuildMarker>,
    pub response: UnboundedSender<Vec<Member>>,
}

struct PendingChunkState {
    received_chunks: u64,
    response: UnboundedSender<Vec<Member>>,
}

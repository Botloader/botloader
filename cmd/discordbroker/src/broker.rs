use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

use dbrokerapi::broker_scheduler_rpc::{self, BrokerEvent, DiscordEventData, HelloData};
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
    voice::VoiceState,
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
        scheduler_disconnected_at: Instant::now(),
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
    queued_events: Vec<(Id<GuildMarker>, BrokerEvent)>,
    scheduler_disconnected_at: Instant,
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
        metrics::counter!("bl.broker.handled_events_total").increment(1);

        match &evt {
            Event::Ready(_) => {
                self.ready.store(true, std::sync::atomic::Ordering::SeqCst);

                metrics::gauge!("bl.broker.connected_guilds_total").set(0.0);
                info!("received ready!");
            }
            Event::GuildDelete(g) => {
                metrics::gauge!("bl.broker.connected_guilds_total").decrement(1.0);

                if !g.unavailable {
                    let _ = self.stores.set_guild_left_status(g.id, true).await;
                }
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
            }
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
            }
            _ => {}
        };

        if let Ok(dispatch) = DispatchEvent::try_from(evt.clone()) {
            if let Some(broker_event) = self.prepare_dispatch_event(dispatch) {
                self.dispatch_or_queue_event(
                    broker_event.guild_id,
                    BrokerEvent::DiscordEvent(broker_event),
                )
                .await;

                metrics::counter!("bl.broker.dispatched_events").increment(1);
            }
        }

        // This is done last as we need the old state on certain events (voice_state_update)
        self.discord_state.update(&evt);
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
            .send_event(&broker_scheduler_rpc::BrokerEvent::Hello(HelloData {
                connected_guilds: guilds,
            }))
            .await
            .is_err()
        {
            self.connected_scheduler = None;
        }

        // send pending events

        let old_queued = std::mem::take(&mut self.queued_events);
        for (guild_id, evt) in old_queued.into_iter() {
            if self.connected_scheduler.is_some() {
                // let prepared = self.prepare_dispatch_event(guild_id, &evt);
                // let v = serde_json::to_value(&evt).unwrap();

                if self.send_event(&evt).await.is_err() {
                    // connection dead, re-queue
                    self.queued_events.push((guild_id, evt));
                    self.connected_scheduler = None;
                    self.scheduler_disconnected_at = Instant::now();
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

    async fn dispatch_or_queue_event(
        &mut self,
        guild_id: Id<GuildMarker>,
        evt: broker_scheduler_rpc::BrokerEvent,
    ) {
        if self.connected_scheduler.is_some() {
            info!("dispatching event");

            // let prepared = self.prepare_dispatch_event(evt);
            if self.send_event(&evt).await.is_err() {
                self.connected_scheduler = None;
                self.scheduler_disconnected_at = Instant::now();
                self.queue_event(guild_id, evt);

                error!("Scheduler disconnected, started queueing events");
            }
        } else {
            info!("queued event");
            self.queue_event(guild_id, evt);
        }
    }

    fn prepare_dispatch_event(
        &mut self,
        evt: DispatchEvent,
    ) -> Option<broker_scheduler_rpc::DiscordEvent> {
        let event_t = evt.kind().name().unwrap().to_string();

        let (guild_id, data) = match evt {
            DispatchEvent::GuildDelete(g) => (g.id, DiscordEventData::GuildDelete(g)),
            DispatchEvent::GuildCreate(gc) => (gc.id, DiscordEventData::GuildCreate(gc)),

            DispatchEvent::MemberAdd(m) => (m.guild_id, DiscordEventData::MemberAdd(m)),
            DispatchEvent::MemberRemove(m) => (m.guild_id, DiscordEventData::MemberRemove(m)),
            DispatchEvent::MemberUpdate(m) => (m.guild_id, DiscordEventData::MemberUpdate(m)),

            DispatchEvent::MessageCreate(m) => (m.guild_id?, DiscordEventData::MessageCreate(m)),
            DispatchEvent::MessageDelete(m) => (m.guild_id?, DiscordEventData::MessageDelete(m)),
            DispatchEvent::MessageDeleteBulk(m) => {
                (m.guild_id?, DiscordEventData::MessageDeleteBulk(m))
            }
            DispatchEvent::MessageUpdate(m) => (m.guild_id?, DiscordEventData::MessageUpdate(m)),

            DispatchEvent::ReactionAdd(r) => (r.guild_id?, DiscordEventData::ReactionAdd(r)),
            DispatchEvent::ReactionRemove(r) => (r.guild_id?, DiscordEventData::ReactionRemove(r)),
            DispatchEvent::ReactionRemoveAll(r) => {
                (r.guild_id?, DiscordEventData::ReactionRemoveAll(r))
            }
            DispatchEvent::ReactionRemoveEmoji(r) => {
                (r.guild_id, DiscordEventData::ReactionRemoveEmoji(r))
            }

            DispatchEvent::InteractionCreate(i) => {
                (i.guild_id?, DiscordEventData::InteractionCreate(i))
            }
            DispatchEvent::ChannelCreate(v) => (v.guild_id?, DiscordEventData::ChannelCreate(v)),
            DispatchEvent::ChannelUpdate(v) => (v.guild_id?, DiscordEventData::ChannelUpdate(v)),
            DispatchEvent::ChannelDelete(v) => (v.guild_id?, DiscordEventData::ChannelDelete(v)),

            DispatchEvent::ThreadCreate(v) => (v.guild_id?, DiscordEventData::ThreadCreate(v)),
            DispatchEvent::ThreadUpdate(v) => (v.guild_id?, DiscordEventData::ThreadUpdate(v)),
            DispatchEvent::ThreadDelete(v) => (v.guild_id, DiscordEventData::ThreadDelete(v)),
            DispatchEvent::ThreadListSync(v) => (v.guild_id, DiscordEventData::ThreadListSync(v)),
            DispatchEvent::ThreadMemberUpdate(v) => {
                (v.guild_id, DiscordEventData::ThreadMemberUpdate(v))
            }
            DispatchEvent::ThreadMembersUpdate(v) => {
                (v.guild_id, DiscordEventData::ThreadMembersUpdate(v))
            }

            DispatchEvent::InviteCreate(invite) => {
                (invite.guild_id, DiscordEventData::InviteCreate(invite))
            }
            DispatchEvent::InviteDelete(invite) => {
                (invite.guild_id, DiscordEventData::InviteDelete(invite))
            }
            DispatchEvent::VoiceStateUpdate(update) => {
                let guild_id = update.guild_id?;

                let old_state = self
                    .discord_state
                    .voice_state(update.user_id, guild_id)
                    .map(|v| {
                        Box::new(VoiceState {
                            channel_id: Some(v.channel_id()),
                            guild_id: Some(guild_id),
                            deaf: v.deaf(),
                            member: None,
                            mute: v.mute(),
                            self_deaf: v.self_deaf(),
                            self_mute: v.self_mute(),
                            self_stream: v.self_stream(),
                            self_video: v.self_video(),
                            session_id: v.session_id().to_owned(),
                            suppress: v.suppress(),
                            user_id: v.user_id(),
                            request_to_speak_timestamp: v.request_to_speak_timestamp(),
                        })
                    });

                (
                    guild_id,
                    DiscordEventData::VoiceStateUpdate {
                        event: update,
                        old_state,
                    },
                )
            }

            _ => return None,
        };
        Some(broker_scheduler_rpc::DiscordEvent {
            t: event_t,
            guild_id,
            event: data,
        })
    }

    fn queue_event(&mut self, guild_id: Id<GuildMarker>, evt: BrokerEvent) {
        if Instant::elapsed(&self.scheduler_disconnected_at) > Duration::from_secs(60) {
            warn!("event queue too old, expired, clearing");
            self.queued_events = Vec::new();
            return;
        }

        self.queued_events.push((guild_id, evt));
    }

    async fn send_event(&mut self, evt: &broker_scheduler_rpc::BrokerEvent) -> std::io::Result<()> {
        if let Some(connected) = &mut self.connected_scheduler {
            simpleproto::write_message(&evt, connected).await?;
            self.wait_for_ack().await?;
        }

        Ok(())
    }

    async fn wait_for_ack(&mut self) -> std::io::Result<()> {
        if let Some(connected) = &mut self.connected_scheduler {
            let _msg: broker_scheduler_rpc::SchedulerEvent =
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

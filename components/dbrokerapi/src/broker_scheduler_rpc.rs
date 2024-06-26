use serde::{Deserialize, Serialize};
use twilight_model::{
    gateway::payload::incoming::{
        ChannelCreate, ChannelDelete, ChannelUpdate, GuildCreate, GuildDelete, InteractionCreate,
        InviteCreate, InviteDelete, MemberAdd, MemberRemove, MemberUpdate, MessageCreate,
        MessageDelete, MessageDeleteBulk, MessageUpdate, ReactionAdd, ReactionRemove,
        ReactionRemoveAll, ReactionRemoveEmoji, RoleCreate, RoleDelete, RoleUpdate, ThreadCreate,
        ThreadDelete, ThreadListSync, ThreadMemberUpdate, ThreadMembersUpdate, ThreadUpdate,
        VoiceStateUpdate, WebhooksUpdate,
    },
    id::{marker::GuildMarker, Id},
    voice::VoiceState,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum BrokerEvent {
    Hello(HelloData),
    DiscordEvent(DiscordEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HelloData {
    pub connected_guilds: Vec<Id<GuildMarker>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordEvent {
    pub t: String,
    pub guild_id: Id<GuildMarker>,
    pub event: DiscordEventData,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DiscordEventData {
    GuildDelete(GuildDelete),
    GuildCreate(Box<GuildCreate>),

    MemberAdd(Box<MemberAdd>),
    MemberRemove(MemberRemove),
    MemberUpdate(Box<MemberUpdate>),

    MessageCreate(Box<MessageCreate>),
    MessageDelete(MessageDelete),
    MessageDeleteBulk(MessageDeleteBulk),
    MessageUpdate(Box<MessageUpdate>),

    ReactionAdd(Box<ReactionAdd>),
    ReactionRemove(Box<ReactionRemove>),
    ReactionRemoveAll(ReactionRemoveAll),
    ReactionRemoveEmoji(ReactionRemoveEmoji),

    InteractionCreate(Box<InteractionCreate>),

    ChannelCreate(Box<ChannelCreate>),
    ChannelUpdate(Box<ChannelUpdate>),
    ChannelDelete(Box<ChannelDelete>),

    RoleCreate(RoleCreate),
    RoleUpdate(RoleUpdate),
    RoleDelete(RoleDelete),

    ThreadCreate(Box<ThreadCreate>),
    ThreadUpdate(Box<ThreadUpdate>),
    ThreadDelete(ThreadDelete),

    ThreadListSync(ThreadListSync),
    ThreadMemberUpdate(Box<ThreadMemberUpdate>),
    ThreadMembersUpdate(ThreadMembersUpdate),

    InviteCreate(Box<InviteCreate>),
    InviteDelete(InviteDelete),

    VoiceStateUpdate {
        event: Box<VoiceStateUpdate>,
        old_state: Option<Box<VoiceState>>,
    },

    WebhooksUpdate(WebhooksUpdate),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SchedulerEvent {
    Ack,
}

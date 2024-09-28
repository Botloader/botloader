use serde::Serialize;
use ts_rs::TS;

use crate::{
    discord::{
        embed::Embed,
        invite::{InviteTargetType, InviteTargetUser},
        message::{Attachment, MessageType, ReactionType},
    },
    internal::{member::Member, messages::UserMention, user::User},
    util::NotBigU64,
};

use super::channel::{GuildChannel, ThreadMember};

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IEventMemberRemove")]
#[ts(export_to = "bindings/internal/EventMemberRemove.ts")]
#[serde(rename_all = "camelCase")]
pub struct EventMemberRemove {
    pub guild_id: String,
    pub user: User,
}

impl From<twilight_model::gateway::payload::incoming::MemberRemove> for EventMemberRemove {
    fn from(v: twilight_model::gateway::payload::incoming::MemberRemove) -> Self {
        Self {
            guild_id: v.guild_id.to_string(),
            user: v.user.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IEventMessageReactionAdd")]
#[ts(export_to = "bindings/internal/EventMessageReactionAdd.ts")]
#[serde(rename_all = "camelCase")]
pub struct EventMessageReactionAdd {
    pub channel_id: String,
    pub message_id: String,
    pub emoji: ReactionType,
    pub member: Member,
    pub user_id: String,
}

impl From<twilight_model::gateway::payload::incoming::ReactionAdd> for EventMessageReactionAdd {
    fn from(v: twilight_model::gateway::payload::incoming::ReactionAdd) -> Self {
        Self {
            channel_id: v.channel_id.to_string(),
            message_id: v.message_id.to_string(),
            emoji: v.0.emoji.into(),
            member: v
                .0
                .member
                .expect("member is always available in guild events")
                .into(),
            user_id: v.0.user_id.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IEventMessageUpdate")]
#[ts(export_to = "bindings/internal/EventMessageUpdate.ts")]
#[serde(rename_all = "camelCase")]
pub struct EventMessageUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<User>,
    pub channel_id: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_timestamp: Option<NotBigU64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<Embed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<String>,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<MessageType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention_everyone: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention_roles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<UserMention>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<NotBigU64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<bool>,
}

impl TryFrom<twilight_model::gateway::payload::incoming::MessageUpdate> for EventMessageUpdate {
    type Error = anyhow::Error;

    fn try_from(
        v: twilight_model::gateway::payload::incoming::MessageUpdate,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            attachments: v
                .attachments
                .map(|e| e.into_iter().map(From::from).collect()),
            author: v.author.map(From::from),
            channel_id: v.channel_id.to_string(),
            content: v.content,
            edited_timestamp: v
                .edited_timestamp
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            embeds: v.embeds.map(|e| e.into_iter().map(From::from).collect()),
            guild_id: v.guild_id.as_ref().map(ToString::to_string),
            id: v.id.to_string(),
            kind: v.kind.map(TryFrom::try_from).transpose()?,
            mention_everyone: v.mention_everyone,
            mention_roles: v
                .mention_roles
                .map(|r| r.iter().map(ToString::to_string).collect()),
            mentions: v.mentions.map(|e| e.into_iter().map(From::from).collect()),
            pinned: v.pinned,
            timestamp: v
                .timestamp
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            tts: v.tts,
        })
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IEventInviteDelete")]
#[ts(export_to = "bindings/internal/IEventInviteDelete.ts")]
#[serde(rename_all = "camelCase")]
pub struct EventInviteDelete {
    pub channel_id: String,
    pub code: String,
}

impl From<twilight_model::gateway::payload::incoming::InviteDelete> for EventInviteDelete {
    fn from(value: twilight_model::gateway::payload::incoming::InviteDelete) -> Self {
        Self {
            channel_id: value.channel_id.to_string(),
            code: value.code,
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IEventInviteCreate")]
#[ts(export_to = "bindings/internal/IEventInviteCreate.ts")]
#[serde(rename_all = "camelCase")]
pub struct EventInviteCreate {
    pub channel_id: String,
    pub code: String,
    pub created_at: NotBigU64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inviter: Option<User>,
    pub max_age: NotBigU64,
    pub max_uses: NotBigU64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_user_type: Option<InviteTargetType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_user: Option<InviteTargetUser>,
    pub temporary: bool,
    pub uses: u8,
}

impl TryFrom<twilight_model::gateway::payload::incoming::InviteCreate> for EventInviteCreate {
    type Error = anyhow::Error;

    fn try_from(
        value: twilight_model::gateway::payload::incoming::InviteCreate,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            channel_id: value.channel_id.to_string(),
            code: value.code,
            created_at: ((value.created_at.as_micros() / 1000) as u64).into(),
            inviter: value.inviter.map(Into::into),
            max_age: value.max_age.into(),
            max_uses: value.max_uses.into(),
            target_user_type: if let Some(t) = value.target_user_type {
                Some(t.try_into()?)
            } else {
                None
            },
            target_user: value.target_user.map(Into::into),
            temporary: value.temporary,
            uses: value.uses,
        })
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IVoiceState")]
#[ts(export_to = "bindings/internal/IVoiceState.ts")]
#[serde(rename_all = "camelCase")]
pub struct VoiceState {
    pub channel_id: Option<String>,
    pub deaf: bool,
    pub guild_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<Member>,
    pub mute: bool,
    pub self_deaf: bool,
    pub self_mute: bool,
    #[serde(default)]
    pub self_stream: bool,
    pub self_video: bool,
    pub session_id: String,
    pub suppress: bool,
    pub user_id: String,
    pub request_to_speak_timestamp: Option<NotBigU64>,
}

impl TryFrom<twilight_model::voice::VoiceState> for VoiceState {
    type Error = anyhow::Error;

    fn try_from(value: twilight_model::voice::VoiceState) -> Result<Self, Self::Error> {
        Ok(Self {
            channel_id: value.channel_id.map(|v| v.to_string()),
            deaf: value.deaf,
            guild_id: value
                .guild_id
                .map(|v| v.to_string())
                .ok_or(anyhow::anyhow!("guild_id is None"))?,
            member: value.member.map(Into::into),
            mute: value.mute,
            self_deaf: value.self_deaf,
            self_mute: value.self_mute,
            self_stream: value.self_stream,
            self_video: value.self_video,
            session_id: value.session_id,
            suppress: value.suppress,
            user_id: value.user_id.to_string(),
            request_to_speak_timestamp: value
                .request_to_speak_timestamp
                .map(|v| ((v.as_micros() / 1000) as u64).into()),
        })
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IEventVoiceStateUpdate")]
#[ts(export_to = "bindings/internal/IEventVoiceStateUpdate.ts")]
#[serde(rename_all = "camelCase")]
pub struct EventVoiceStateUpdate {
    pub new: VoiceState,
    pub old: Option<VoiceState>,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IEventThreadListSync")]
#[ts(export_to = "bindings/internal/IEventThreadListSync.ts")]
#[serde(rename_all = "camelCase")]
pub struct EventThreadListSync {
    pub channel_ids: Vec<String>,
    pub members: Vec<ThreadMember>,
    pub threads: Vec<GuildChannel>,
}

impl TryFrom<twilight_model::gateway::payload::incoming::ThreadListSync> for EventThreadListSync {
    type Error = anyhow::Error;

    fn try_from(
        value: twilight_model::gateway::payload::incoming::ThreadListSync,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            channel_ids: value
                .channel_ids
                .into_iter()
                .map(|v| v.to_string())
                .collect(),
            members: value.members.into_iter().map(Into::into).collect(),
            threads: value
                .threads
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IEventThreadMembersUpdate")]
#[ts(export_to = "bindings/internal/IEventThreadMembersUpdate.ts")]
#[serde(rename_all = "camelCase")]
pub struct EventThreadMembersUpdate {
    pub added_members: Vec<ThreadMember>,
    pub id: String,
    pub member_count: i32,
    pub removed_member_ids: Vec<String>,
}

impl From<twilight_model::gateway::payload::incoming::ThreadMembersUpdate>
    for EventThreadMembersUpdate
{
    fn from(value: twilight_model::gateway::payload::incoming::ThreadMembersUpdate) -> Self {
        Self {
            added_members: value.added_members.into_iter().map(Into::into).collect(),
            id: value.id.to_string(),
            member_count: value.member_count,
            removed_member_ids: value
                .removed_member_ids
                .into_iter()
                .map(|v| v.to_string())
                .collect(),
        }
    }
}

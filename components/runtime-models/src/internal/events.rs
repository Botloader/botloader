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

impl From<twilight_model::gateway::payload::incoming::MessageUpdate> for EventMessageUpdate {
    fn from(v: twilight_model::gateway::payload::incoming::MessageUpdate) -> Self {
        Self {
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
            kind: v.kind.map(From::from),
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
        }
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
#[ts(export, rename = "IEventVoiceStateUpdate")]
#[ts(export_to = "bindings/internal/IEventVoiceStateUpdate.ts")]
#[serde(rename_all = "camelCase")]
pub struct EventVoiceStateUpdate {
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

impl TryFrom<twilight_model::gateway::payload::incoming::VoiceStateUpdate>
    for EventVoiceStateUpdate
{
    type Error = anyhow::Error;

    fn try_from(
        value: twilight_model::gateway::payload::incoming::VoiceStateUpdate,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            channel_id: value.0.channel_id.map(|v| v.to_string()),
            deaf: value.0.deaf,
            guild_id: value
                .0
                .guild_id
                .map(|v| v.to_string())
                .ok_or(anyhow::anyhow!("guild_id is None"))?,
            member: value.0.member.map(Into::into),
            mute: value.0.mute,
            self_deaf: value.0.self_deaf,
            self_mute: value.0.self_mute,
            self_stream: value.0.self_stream,
            self_video: value.0.self_video,
            session_id: value.0.session_id,
            suppress: value.0.suppress,
            user_id: value.0.user_id.to_string(),
            request_to_speak_timestamp: value
                .0
                .request_to_speak_timestamp
                .map(|v| ((v.as_micros() / 1000) as u64).into()),
        })
    }
}

use serde::Serialize;
use ts_rs::TS;

use crate::{
    discord::{
        embed::Embed,
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

use serde::Serialize;
use ts_rs::TS;

use crate::discord::message::ReactionType;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/EventMessageDelete.ts")]
#[serde(rename_all = "camelCase")]
pub struct EventMessageDelete {
    pub channel_id: String,
    pub id: String,
}

impl From<twilight_model::gateway::payload::incoming::MessageDelete> for EventMessageDelete {
    fn from(v: twilight_model::gateway::payload::incoming::MessageDelete) -> Self {
        Self {
            channel_id: v.channel_id.to_string(),
            id: v.id.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/EventMessageReactionRemove.ts")]
#[serde(rename_all = "camelCase")]
pub struct EventMessageReactionRemove {
    pub channel_id: String,
    pub message_id: String,
    pub emoji: ReactionType,
    pub user_id: String,
}

impl From<twilight_model::gateway::payload::incoming::ReactionRemove>
    for EventMessageReactionRemove
{
    fn from(v: twilight_model::gateway::payload::incoming::ReactionRemove) -> Self {
        Self {
            channel_id: v.channel_id.to_string(),
            message_id: v.message_id.to_string(),
            emoji: v.0.emoji.into(),
            user_id: v.0.user_id.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/EventMessageReactionRemoveAll.ts")]
#[serde(rename_all = "camelCase")]
pub struct EventMessageReactionRemoveAll {
    pub channel_id: String,
    pub message_id: String,
}

impl From<twilight_model::gateway::payload::incoming::ReactionRemoveAll>
    for EventMessageReactionRemoveAll
{
    fn from(v: twilight_model::gateway::payload::incoming::ReactionRemoveAll) -> Self {
        Self {
            channel_id: v.channel_id.to_string(),
            message_id: v.message_id.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/EventMessageReactionRemoveAllEmoji.ts")]
#[serde(rename_all = "camelCase")]
pub struct EventMessageReactionRemoveAllEmoji {
    pub channel_id: String,
    pub message_id: String,
    pub emoji: ReactionType,
}

impl From<twilight_model::gateway::payload::incoming::ReactionRemoveEmoji>
    for EventMessageReactionRemoveAllEmoji
{
    fn from(v: twilight_model::gateway::payload::incoming::ReactionRemoveEmoji) -> Self {
        Self {
            channel_id: v.channel_id.to_string(),
            message_id: v.message_id.to_string(),
            emoji: v.emoji.into(),
        }
    }
}

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    discord::{
        channel::{ChannelType, ThreadMetadata},
        message::MessageFlags,
    },
    util::NotBigU64,
};
use twilight_model::application::interaction::application_command;

use super::messages::OpCreateMessageFields;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/InteractionChannel.ts")]
#[serde(rename_all = "camelCase")]
pub struct InteractionPartialChannel {
    pub id: String,
    pub kind: ChannelType,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub permissions_raw: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_metadata: Option<ThreadMetadata>,
}

impl From<application_command::InteractionChannel> for InteractionPartialChannel {
    fn from(v: application_command::InteractionChannel) -> Self {
        Self {
            id: v.id.to_string(),
            kind: v.kind.into(),
            name: v.name,
            parent_id: v.parent_id.as_ref().map(ToString::to_string),
            permissions_raw: v.permissions.bits().to_string(),
            thread_metadata: v.thread_metadata.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/InteractionPartialMember.ts")]
#[serde(rename_all = "camelCase")]
pub struct InteractionPartialMember {
    pub joined_at: NotBigU64,
    pub nick: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub premium_since: Option<NotBigU64>,
    #[serde(default)]
    pub roles: Vec<String>,
}

impl From<application_command::InteractionMember> for InteractionPartialMember {
    fn from(v: application_command::InteractionMember) -> Self {
        Self {
            joined_at: NotBigU64(v.joined_at.as_micros() as u64 / 1000),
            nick: v.nick,
            premium_since: v
                .premium_since
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            roles: v.roles.iter().map(ToString::to_string).collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/InteractionCallback.ts")]
#[serde(rename_all = "camelCase")]
pub struct InteractionCallback {
    pub interaction_id: String,
    pub ineraction_token: String,
    pub data: InteractionResponse,
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/InteractionResponse.ts")]
#[serde(tag = "kind")]
pub enum InteractionResponse {
    Pong,
    ChannelMessageWithSource(InteractionCallbackData),
    DeferredChannelMessageWithSource(InteractionCallbackData),
    DeferredUpdateMessage,
    UpdateMessage(InteractionCallbackData),
    // Autocomplete(Autocomplete),
}

impl From<InteractionResponse> for twilight_model::application::callback::InteractionResponse {
    fn from(v: InteractionResponse) -> Self {
        match v {
            InteractionResponse::Pong => Self::Pong,
            InteractionResponse::ChannelMessageWithSource(src) => {
                Self::ChannelMessageWithSource(src.into())
            }
            InteractionResponse::DeferredChannelMessageWithSource(src) => {
                Self::DeferredChannelMessageWithSource(src.into())
            }
            InteractionResponse::DeferredUpdateMessage => Self::DeferredUpdateMessage,
            InteractionResponse::UpdateMessage(src) => Self::UpdateMessage(src.into()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/InteractionCallbackData.ts")]
#[serde(rename_all = "camelCase")]
pub struct InteractionCallbackData {
    pub fields: OpCreateMessageFields,
    pub flags: Option<MessageFlags>,
}

use twilight_model::application::callback::CallbackData as TwilightCallbackData;
impl From<InteractionCallbackData> for TwilightCallbackData {
    fn from(v: InteractionCallbackData) -> Self {
        Self {
            allowed_mentions: v.fields.allowed_mentions.map(Into::into),
            components: v
                .fields
                .components
                .map(|v| v.into_iter().map(Into::into).collect()),
            content: v.fields.content,
            embeds: v
                .fields
                .embeds
                .map(|v| v.into_iter().map(Into::into).collect()),
            flags: v.flags.map(Into::into),
            tts: None,
        }
    }
}

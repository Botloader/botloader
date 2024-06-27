use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{messages::OpCreateMessageFields, user::User};

#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "bindings/internal/DiscordWebhook.ts")]
pub struct DiscordWebhook {
    pub id: String,
    pub application_id: Option<String>,
    pub avatar: Option<String>,
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub kind: WebhookType,
    pub name: Option<String>,
    pub source_channel: Option<WebhookChannel>,
    pub source_guild: Option<WebhookGuild>,
    pub token: Option<String>,
    pub url: Option<String>,
    pub user: Option<User>,
}

impl From<twilight_model::channel::Webhook> for DiscordWebhook {
    fn from(value: twilight_model::channel::Webhook) -> Self {
        Self {
            id: value.id.to_string(),
            application_id: value.application_id.as_ref().map(ToString::to_string),
            avatar: value.avatar.as_ref().map(ToString::to_string),
            channel_id: value.channel_id.to_string(),
            guild_id: value.guild_id.as_ref().map(ToString::to_string),
            kind: value.kind.into(),
            name: value.name,
            source_channel: value.source_channel.map(Into::into),
            source_guild: value.source_guild.map(Into::into),
            token: value.token,
            url: value.url,
            user: value.user.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, export_to = "bindings/internal/WebhookType.ts")]
pub enum WebhookType {
    Incoming,
    ChannelFollower,
    Application,
    Unknown,
}

impl From<twilight_model::channel::WebhookType> for WebhookType {
    fn from(value: twilight_model::channel::WebhookType) -> Self {
        match value {
            twilight_model::channel::WebhookType::Incoming => Self::Incoming,
            twilight_model::channel::WebhookType::ChannelFollower => Self::ChannelFollower,
            twilight_model::channel::WebhookType::Application => Self::Application,
            twilight_model::channel::WebhookType::Unknown(t) => {
                tracing::warn!("Unknown webhook type {t}");
                Self::Unknown
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, export_to = "bindings/internal/WebhookChannel.ts")]
pub struct WebhookChannel {
    pub id: String,
    pub name: String,
}

impl From<twilight_model::channel::webhook::WebhookChannel> for WebhookChannel {
    fn from(value: twilight_model::channel::webhook::WebhookChannel) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, export_to = "bindings/internal/WebhookGuild.ts")]
pub struct WebhookGuild {
    pub icon: Option<String>,
    pub id: String,
    pub name: String,
}

impl From<twilight_model::channel::webhook::WebhookGuild> for WebhookGuild {
    fn from(value: twilight_model::channel::webhook::WebhookGuild) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            icon: value.icon.as_ref().map(ToString::to_string),
        }
    }
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export, export_to = "bindings/internal/OpCreateWebhook.ts")]
pub struct OpCreateWebhook {
    pub icon: Option<String>,
    pub name: String,
    pub channel_id: String,
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export, export_to = "bindings/internal/OpEditWebhook.ts")]
pub struct OpEditWebhook {
    pub webhook_id: String,

    #[serde(
        deserialize_with = "crate::deserialize_undefined_null_optional_field",
        default
    )]
    #[ts(optional)]
    pub icon: Option<Option<String>>,
    pub channel_id: Option<String>,
    pub name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export, export_to = "bindings/internal/OpEditWebhookWithToken.ts")]
pub struct OpEditWebhookWithToken {
    pub webhook_id: String,

    #[serde(
        deserialize_with = "crate::deserialize_undefined_null_optional_field",
        default
    )]
    #[ts(optional)]
    pub icon: Option<Option<String>>,
    pub name: Option<String>,
    pub token: String,
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export, export_to = "bindings/internal/OpExecuteWebhook.ts")]
pub struct OpExecuteWebhook {
    pub webhook_id: String,
    pub token: String,

    pub fields: OpCreateMessageFields,
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export, export_to = "bindings/internal/OpWebhookMessageSpecifier.ts")]
pub struct OpWebhookMessageSpecifier {
    pub webhook_id: String,
    pub token: String,
    pub message_id: String,
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export, export_to = "bindings/internal/OpWebhookSpecifier.ts")]
pub struct OpWebhookSpecifier {
    pub webhook_id: String,
    pub token: Option<String>,
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export, export_to = "bindings/internal/OpUpdateWebhookMessage.ts")]
pub struct OpUpdateWebhookMessage {
    pub webhook_id: String,
    pub token: String,
    pub message_id: String,

    pub fields: OpCreateMessageFields,
}

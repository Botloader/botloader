use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::util::NotBigU64;

use super::{channel::ChannelType, guild::VerificationLevel};

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IInviteChannel")]
#[ts(export_to = "bindings/discord/IInviteChannel.ts")]
#[serde(rename_all = "camelCase")]
pub struct InviteChannel {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub kind: ChannelType,
}

impl From<twilight_model::guild::invite::InviteChannel> for InviteChannel {
    fn from(value: twilight_model::guild::invite::InviteChannel) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            kind: value.kind.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IInviteGuild")]
#[ts(export_to = "bindings/discord/IInviteGuild.ts")]
#[serde(rename_all = "camelCase")]
pub struct InviteGuild {
    pub banner: Option<String>,
    pub description: Option<String>,
    pub features: Vec<String>,
    pub icon: Option<String>,
    pub id: String,
    pub name: String,
    pub premium_subscription_count: Option<NotBigU64>,
    pub splash: Option<String>,
    pub vanity_url_code: Option<String>,
    pub verification_level: VerificationLevel,
    // pub welcome_screen: Option<WelcomeScreen>,
}

impl From<twilight_model::guild::invite::InviteGuild> for InviteGuild {
    fn from(value: twilight_model::guild::invite::InviteGuild) -> Self {
        Self {
            banner: value.banner.map(|v| v.to_string()),
            description: value.description,
            features: value
                .features
                .into_iter()
                .map(|v| Cow::<str>::from(v).to_string())
                .collect(),
            icon: value.icon.map(|v| v.to_string()),
            id: value.id.to_string(),
            name: value.name,
            premium_subscription_count: value.premium_subscription_count.map(Into::into),
            splash: value.splash.map(|v| v.to_string()),
            vanity_url_code: value.vanity_url_code,
            verification_level: value.verification_level.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, rename = "InviteTargetType")]
#[ts(export_to = "bindings/discord/InviteTargetType.ts")]
pub enum InviteTargetType {
    Stream,
    EmbeddedApplication,
}

impl TryFrom<twilight_model::guild::invite::TargetType> for InviteTargetType {
    type Error = anyhow::Error;

    fn try_from(value: twilight_model::guild::invite::TargetType) -> Result<Self, Self::Error> {
        match value {
            twilight_model::guild::invite::TargetType::Stream => Ok(Self::Stream),
            twilight_model::guild::invite::TargetType::EmbeddedApplication => {
                Ok(Self::EmbeddedApplication)
            }

            other => {
                let kind = u8::from(other);
                Err(anyhow::anyhow!(
                    "unimplemented invite target type: {:?}",
                    kind
                ))
            }
        }
    }
}

impl From<InviteTargetType> for twilight_model::guild::invite::TargetType {
    fn from(value: InviteTargetType) -> Self {
        match value {
            InviteTargetType::Stream => Self::Stream,
            InviteTargetType::EmbeddedApplication => Self::EmbeddedApplication,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export, rename = "IInviteTargetUser")]
#[ts(export_to = "bindings/discord/IInviteTargetUser.ts")]
#[serde(rename_all = "camelCase")]
pub struct InviteTargetUser {
    pub avatar: Option<String>,
    pub discriminator: String,
    pub id: String,
    pub username: String,
}

impl From<twilight_model::gateway::payload::incoming::invite_create::PartialUser>
    for InviteTargetUser
{
    fn from(value: twilight_model::gateway::payload::incoming::invite_create::PartialUser) -> Self {
        Self {
            avatar: value.avatar.map(|v| v.to_string()),
            discriminator: value.discriminator.to_string(),
            id: value.id.to_string(),
            username: value.username,
        }
    }
}

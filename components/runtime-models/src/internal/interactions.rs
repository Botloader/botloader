use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    discord::{
        channel::{ChannelType, ThreadMetadata},
        component::Component,
        message::MessageFlags,
    },
    util::NotBigU64,
};

use super::{
    messages::{convert_attachments, OpCreateMessageFields},
    script::CommandOptionChoice,
};

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/InteractionChannel.ts")]
#[serde(rename_all = "camelCase")]
pub struct InteractionPartialChannel {
    pub id: String,
    pub kind: ChannelType,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub parent_id: Option<String>,
    pub permissions_raw: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub thread_metadata: Option<ThreadMetadata>,
}

impl TryFrom<twilight_model::application::interaction::InteractionChannel>
    for InteractionPartialChannel
{
    type Error = anyhow::Error;

    fn try_from(
        v: twilight_model::application::interaction::InteractionChannel,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id.to_string(),
            kind: v.kind.try_into()?,
            name: v.name,
            parent_id: v.parent_id.as_ref().map(ToString::to_string),
            permissions_raw: v.permissions.bits().to_string(),
            thread_metadata: v.thread_metadata.map(Into::into),
        })
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/InteractionPartialMember.ts")]
#[serde(rename_all = "camelCase")]
pub struct InteractionPartialMember {
    pub joined_at: NotBigU64,
    pub nick: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub premium_since: Option<NotBigU64>,
    #[serde(default)]
    pub roles: Vec<String>,
}

impl From<twilight_model::application::interaction::InteractionMember>
    for InteractionPartialMember
{
    fn from(v: twilight_model::application::interaction::InteractionMember) -> Self {
        Self {
            joined_at: NotBigU64(
                v.joined_at
                    .unwrap_or(Timestamp::from_micros(0).unwrap())
                    .as_micros() as u64
                    / 1000,
            ),
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
    pub interaction_token: String,
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
    Modal(ModalCallbackData),
    Autocomplete(AutocompleteCallbackData),
}

impl TryFrom<InteractionResponse> for twilight_model::http::interaction::InteractionResponse {
    type Error = anyhow::Error;

    fn try_from(v: InteractionResponse) -> Result<Self, Self::Error> {
        use twilight_model::http::interaction::InteractionResponseType as TwilightInteractionResponseType;
        Ok(match v {
            InteractionResponse::Pong => Self {
                kind: TwilightInteractionResponseType::Pong,
                data: None,
            },
            InteractionResponse::ChannelMessageWithSource(src) => Self {
                kind: TwilightInteractionResponseType::ChannelMessageWithSource,
                data: Some(src.try_into()?),
            },
            InteractionResponse::DeferredChannelMessageWithSource(src) => Self {
                kind: TwilightInteractionResponseType::DeferredChannelMessageWithSource,
                data: Some(src.try_into()?),
            },
            InteractionResponse::DeferredUpdateMessage => Self {
                kind: TwilightInteractionResponseType::DeferredUpdateMessage,
                data: None,
            },
            InteractionResponse::UpdateMessage(src) => Self {
                kind: TwilightInteractionResponseType::UpdateMessage,
                data: Some(src.try_into()?),
            },
            InteractionResponse::Modal(src) => Self {
                kind: TwilightInteractionResponseType::Modal,
                data: Some(src.try_into()?),
            },
            InteractionResponse::Autocomplete(data) => Self {
                kind: TwilightInteractionResponseType::ApplicationCommandAutocompleteResult,
                data: Some(data.into()),
            },
        })
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

use twilight_model::{
    http::interaction::InteractionResponseData as TwilightCallbackData, util::Timestamp,
};
impl TryFrom<InteractionCallbackData> for TwilightCallbackData {
    type Error = anyhow::Error;

    fn try_from(v: InteractionCallbackData) -> Result<Self, Self::Error> {
        Ok(Self {
            allowed_mentions: v.fields.allowed_mentions.map(Into::into),
            components: v
                .fields
                .components
                .map(|v| {
                    v.into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<_, _>>()
                })
                .transpose()?,
            content: v.fields.content,
            embeds: v
                .fields
                .embeds
                .map(|v| v.into_iter().map(Into::into).collect()),
            flags: v.flags.map(Into::into),
            tts: None,

            attachments: v.fields.attachments.map(convert_attachments).transpose()?,
            choices: None,
            custom_id: None,
            title: None,
            poll: None,
        })
    }
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "bindings/internal/IModalCallbackData.ts",
    rename = "IModalCallbackData"
)]
#[serde(rename_all = "camelCase")]
pub struct ModalCallbackData {
    title: String,
    custom_id: String,
    components: Vec<Component>,
}

impl TryFrom<ModalCallbackData> for TwilightCallbackData {
    type Error = anyhow::Error;

    fn try_from(v: ModalCallbackData) -> Result<Self, Self::Error> {
        Ok(Self {
            title: Some(v.title),
            custom_id: Some(v.custom_id),
            components: Some(
                v.components
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
            ),
            ..Default::default()
        })
    }
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/AutocompleteCallbackData.ts")]
#[serde(rename_all = "camelCase")]
pub struct AutocompleteCallbackData {
    pub choices: Vec<CommandOptionChoice>,
}

impl From<AutocompleteCallbackData> for TwilightCallbackData {
    fn from(v: AutocompleteCallbackData) -> Self {
        Self {
            choices: Some(v.choices.into_iter().map(Into::into).collect()),
            ..Default::default()
        }
    }
}

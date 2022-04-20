use crate::discord::component::ComponentType;

use super::{member::Member, messages::Message};
use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "bindings/internal/IModalInteraction.ts",
    rename = "IModalInteraction"
)]
#[serde(rename_all = "camelCase")]
pub struct ModalInteraction {
    pub channel_id: String,
    pub guild_locale: Option<String>,
    pub id: String,
    pub locale: String,
    pub member: Member,
    pub message: Option<Message>,
    pub token: String,

    pub custom_id: String,
    pub values: Vec<ModalInteractionDataComponent>,
}

use twilight_model::application::interaction::modal::ModalSubmitInteraction as TwilightModalInteraction;
impl From<TwilightModalInteraction> for ModalInteraction {
    fn from(v: TwilightModalInteraction) -> Self {
        Self {
            channel_id: v.channel_id.to_string(),
            guild_locale: v.guild_locale,
            id: v.id.to_string(),
            locale: v.locale,
            member: Member::from_partial(v.member.unwrap()),
            message: v.message.map(Into::into),
            token: v.token,
            custom_id: v.data.custom_id,
            values: v
                .data
                .components
                .into_iter()
                .map(|row| {
                    row.components
                        .into_iter()
                        .map(ModalInteractionDataComponent::from)
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect::<Vec<_>>(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "bindings/internal/IModalInteractionDataComponent.ts",
    rename = "IModalInteractionDataComponent"
)]
#[serde(rename_all = "camelCase")]
pub struct ModalInteractionDataComponent {
    pub custom_id: String,
    pub kind: ComponentType,
    pub value: String,
}

use twilight_model::application::interaction::modal::ModalInteractionDataComponent as TwilightModalInteractionDataComponent;
impl From<TwilightModalInteractionDataComponent> for ModalInteractionDataComponent {
    fn from(v: TwilightModalInteractionDataComponent) -> Self {
        Self {
            custom_id: v.custom_id,
            kind: v.kind.into(),
            value: v.value,
        }
    }
}

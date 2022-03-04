use crate::discord::{component::ComponentType, member::Member, message::Message};
use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, export_to = "bindings/internal/MessageComponentInteraction.ts")]
#[serde(rename_all = "camelCase")]
pub struct MessageComponentInteraction {
    pub channel_id: String,
    pub guild_locale: Option<String>,
    pub id: String,
    pub locale: String,
    pub member: Member,
    pub message: Message,
    pub token: String,

    pub custom_id: String,
    pub component_type: ComponentType,
    pub values: Vec<String>,
}
use twilight_model::application::interaction::MessageComponentInteraction as TwilightComponentInteraction;
impl From<TwilightComponentInteraction> for MessageComponentInteraction {
    fn from(v: TwilightComponentInteraction) -> Self {
        Self {
            channel_id: v.channel_id.to_string(),
            guild_locale: v.guild_locale,
            id: v.id.to_string(),
            locale: v.locale,
            member: Member::from_partial(v.member.unwrap()),
            message: v.message.into(),
            token: v.token,
            custom_id: v.data.custom_id,
            component_type: v.data.component_type.into(),
            values: v.data.values,
        }
    }
}

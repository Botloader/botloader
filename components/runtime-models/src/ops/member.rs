use serde::Deserialize;
use ts_rs::TS;
use twilight_model::id::{ChannelId, RoleId};

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/ops/UpdateGuildMemberFields.ts")]
#[serde(rename_all = "camelCase")]
pub struct UpdateGuildMemberFields {
    #[ts(optional)]
    #[ts(type = "string|null")]
    #[serde(deserialize_with = "crate::deserialize_optional_field")]
    pub channel_id: Option<Option<ChannelId>>,

    #[ts(optional)]
    pub deaf: Option<bool>,

    #[ts(optional)]
    pub mute: Option<bool>,

    #[ts(optional)]
    #[serde(deserialize_with = "crate::deserialize_optional_field")]
    pub nick: Option<Option<String>>,

    #[ts(optional)]
    #[ts(type = "string[]")]
    pub roles: Option<Vec<RoleId>>,
}

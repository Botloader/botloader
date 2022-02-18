use serde::Deserialize;
use ts_rs::TS;
use twilight_model::id::{
    marker::{ChannelMarker, RoleMarker},
    Id,
};

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/UpdateGuildMemberFields.ts")]
#[serde(rename_all = "camelCase")]
pub struct UpdateGuildMemberFields {
    #[ts(optional)]
    #[ts(type = "string|null")]
    #[serde(deserialize_with = "crate::deserialize_optional_field")]
    pub channel_id: Option<Option<Id<ChannelMarker>>>,

    #[ts(optional)]
    pub deaf: Option<bool>,

    #[ts(optional)]
    pub mute: Option<bool>,

    #[ts(optional)]
    #[serde(deserialize_with = "crate::deserialize_optional_field")]
    pub nick: Option<Option<String>>,

    #[ts(optional)]
    #[ts(type = "string[]")]
    pub roles: Option<Vec<Id<RoleMarker>>>,
}

use serde::Deserialize;
use ts_rs::TS;

use crate::discord::role::RoleColors;

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/CreateRoleFields.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpCreateRoleFields {
    #[ts(optional)]
    pub color: Option<u32>,

    #[ts(optional)]
    pub colors: Option<RoleColors>,

    #[ts(optional)]
    pub hoist: Option<bool>,

    #[ts(optional)]
    pub icon: Option<String>,

    #[ts(optional)]
    pub mentionable: Option<bool>,

    #[ts(optional)]
    pub name: Option<String>,

    #[ts(optional)]
    pub permissions: Option<String>,

    #[ts(optional)]
    pub unicode_emoji: Option<String>,
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/UpdateRoleFields.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpUpdateRoleFields {
    pub role_id: String,

    #[serde(
        deserialize_with = "crate::deserialize_undefined_null_optional_field",
        default
    )]
    #[ts(optional)]
    pub color: Option<Option<u32>>,

    #[ts(optional)]
    pub colors: Option<Option<RoleColors>>,

    #[ts(optional)]
    pub hoist: Option<bool>,
<<<<<<< HEAD
    
    #[serde(
        deserialize_with = "crate::deserialize_undefined_null_optional_field",
        default
    )]
    #[ts(optional)]
    pub icon: Option<Option<String>>,
=======
>>>>>>> upstream/master

    #[ts(optional)]
    pub mentionable: Option<bool>,

    #[serde(
        deserialize_with = "crate::deserialize_undefined_null_optional_field",
        default
    )]
    #[ts(optional)]
    pub name: Option<Option<String>>,

    #[ts(optional)]
    pub permissions: Option<String>,

    #[serde(
        deserialize_with = "crate::deserialize_undefined_null_optional_field",
        default
    )]
    #[ts(optional)]
    pub unicode_emoji: Option<Option<String>>,
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(
    export,
    rename = "UpdateRolePosition",
    export_to = "bindings/internal/UpdateRolePosition.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRolePosition {
    pub role_id: String,
    pub position: i32,
}

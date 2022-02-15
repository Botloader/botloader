use serde::Serialize;
use ts_rs::TS;
use twilight_model::guild::{Role as TwilightRole, RoleTags as TwilightRoleTags};

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/Role.ts")]
pub struct Role {
    pub(crate) color: u32,
    pub(crate) hoist: bool,
    pub(crate) icon: Option<String>,
    pub(crate) id: String,
    pub(crate) managed: bool,
    pub(crate) mentionable: bool,
    pub(crate) name: String,
    pub(crate) permissions: String,
    pub(crate) position: i64,
    pub(crate) tags: Option<RoleTags>,
    pub(crate) unicode_emoji: Option<String>,
}

impl From<&TwilightRole> for Role {
    fn from(v: &TwilightRole) -> Self {
        Self {
            color: v.color,
            hoist: v.hoist,
            icon: v.icon.as_ref().map(ToString::to_string),
            id: v.id.to_string(),
            managed: v.managed,
            mentionable: v.mentionable,
            name: v.name.clone(),
            permissions: v.permissions.bits().to_string(),
            position: v.position,
            tags: v.tags.clone().map(Into::into),
            unicode_emoji: v.unicode_emoji.clone(),
        }
    }
}

impl From<TwilightRole> for Role {
    fn from(v: TwilightRole) -> Self {
        Self {
            color: v.color,
            hoist: v.hoist,
            icon: v.icon.as_ref().map(ToString::to_string),
            id: v.id.to_string(),
            managed: v.managed,
            mentionable: v.mentionable,
            name: v.name,
            permissions: v.permissions.bits().to_string(),
            position: v.position,
            tags: v.tags.map(Into::into),
            unicode_emoji: v.unicode_emoji,
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/RoleTags.ts")]
pub struct RoleTags {
    pub(crate) bot_id: Option<String>,
    pub(crate) integration_id: Option<String>,
    pub(crate) premium_subscriber: bool,
}

impl From<TwilightRoleTags> for RoleTags {
    fn from(v: TwilightRoleTags) -> Self {
        Self {
            bot_id: v.bot_id.as_ref().map(ToString::to_string),
            integration_id: v.integration_id.as_ref().map(ToString::to_string),
            premium_subscriber: v.premium_subscriber,
        }
    }
}

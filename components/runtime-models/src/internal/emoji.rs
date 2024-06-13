use dbrokerapi::models::BrokerEmoji;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use twilight_model::guild::Emoji;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/CustomEmoji.ts")]
#[serde(rename_all = "camelCase")]
pub struct CustomEmoji {
    pub animated: bool,
    pub available: bool,
    pub id: String,
    pub managed: bool,
    pub name: String,
    pub require_colons: bool,
    pub roles: Vec<String>,
    pub created_by_user_id: Option<String>,
}

impl From<BrokerEmoji> for CustomEmoji {
    fn from(value: BrokerEmoji) -> Self {
        Self {
            animated: value.animated,
            available: value.available,
            id: value.id.to_string(),
            managed: value.managed,
            name: value.name,
            require_colons: value.require_colons,
            roles: value.roles.into_iter().map(|v| v.to_string()).collect(),
            created_by_user_id: value.user_id.map(|v| v.to_string()),
        }
    }
}

impl From<Emoji> for CustomEmoji {
    fn from(value: Emoji) -> Self {
        Self {
            animated: value.animated,
            available: value.available,
            id: value.id.to_string(),
            managed: value.managed,
            name: value.name,
            require_colons: value.require_colons,
            roles: value.roles.into_iter().map(|v| v.to_string()).collect(),
            created_by_user_id: value.user.map(|v| v.id.to_string()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/OpCreateEmoji.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpCreateEmoji {
    pub name: String,
    // base64 image data uri
    pub data: String,
    pub roles: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/OpUpdateEmoji.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpUpdateEmoji {
    pub id: String,
    pub name: Option<String>,
    pub roles: Option<Vec<String>>,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use twilight_model::id::{marker::UserMarker, Id};
use uuid::Uuid;

#[derive(Serialize, Clone)]
pub struct Plugin {
    pub id: u64,
    pub created_at: DateTime<Utc>,
    pub author_id: Id<UserMarker>,
    pub name: String,
    pub short_description: String,
    pub long_description: String,
    pub is_public: bool,
    pub is_official: bool,
    pub is_published: bool,
    pub current_version: u32,

    pub images: Vec<PluginImage>,

    pub data: PluginData,
}

#[derive(Serialize, Clone)]
#[serde(tag = "plugin_type")]
pub enum PluginData {
    ScriptPlugin(ScriptPluginData),
}

impl PluginData {
    pub fn kind(&self) -> PluginType {
        match self {
            PluginData::ScriptPlugin(_) => PluginType::Script,
        }
    }
}

pub enum PluginType {
    Script = 0,
}

#[derive(Serialize, Clone)]
pub struct ScriptPluginData {
    pub published_version: Option<String>,
    pub published_version_updated_at: Option<DateTime<Utc>>,
    pub dev_version: Option<String>,
    pub dev_version_updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum PluginImageKind {
    Icon,
    Banner,
    Showcase,
}

impl From<i32> for PluginImageKind {
    fn from(value: i32) -> Self {
        match value {
            1 => PluginImageKind::Icon,
            2 => PluginImageKind::Banner,
            3 => PluginImageKind::Showcase,
            _ => PluginImageKind::Showcase,
        }
    }
}

impl From<PluginImageKind> for i32 {
    fn from(value: PluginImageKind) -> Self {
        match value {
            PluginImageKind::Icon => 1,
            PluginImageKind::Banner => 2,
            PluginImageKind::Showcase => 3,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct PluginImage {
    pub plugin_id: u64,
    pub image_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub description: String,
    pub position: i32,
    pub kind: PluginImageKind,
    pub width: u32,
    pub height: u32,
}

pub struct Image {
    pub id: Uuid,
    pub uploaded_by: u64,
    pub plugin_id: Option<u64>,
    pub width: u32,
    pub height: u32,
    pub bytes: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// pub struct Version {
//     pub created_at: DateTime<Utc>,
//     pub kind: VersionKind,
//     pub number: VersionNumber,
//     pub data: VersionData,
// }

// pub struct VersionData {
//     pub meta: VersionMeta,
//     pub sources: HashMap<String, String>,
// }

// pub enum VersionKind {
//     Stable,
//     PreRelease,
//     Development,
// }

// pub struct VersionNumber {
//     pub major: u16,
//     pub minor: u16,
// }

// pub struct VersionMeta {}

// pub struct GuildPluginSubscription {
//     pub guild_id: Id<GuildMarker>,
//     pub plugin_id: u64,
//     pub version: VersionSelector,
// }

// pub enum VersionSelector {
//     LatestStable,
//     LatestDevel,
//     Pinned(VersionNumber),
// }

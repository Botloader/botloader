use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct Plugin {
    pub id: u64,
    pub created_at: DateTime<Utc>,
    pub author_id: u64,
    pub name: String,
    pub short_description: String,
    pub long_description: String,
    pub is_public: bool,
    pub is_official: bool,

    pub data: PluginData,
}

#[derive(Serialize)]
pub enum PluginData {
    ScriptPluginData(ScriptPluginData),
}

impl PluginData {
    pub fn kind(&self) -> PluginType {
        match self {
            PluginData::ScriptPluginData(_) => PluginType::Script,
        }
    }
}

pub enum PluginType {
    Script = 0,
}

#[derive(Serialize)]
pub struct ScriptPluginData {
    pub published_version: Option<String>,
    pub published_version_updated_at: Option<DateTime<Utc>>,
    pub dev_version: Option<String>,
    pub dev_version_updated_at: Option<DateTime<Utc>>,
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

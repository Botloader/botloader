use std::{collections::HashMap, num::NonZeroU64};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use twilight_model::id::{marker::GuildMarker, Id};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: u64,
    pub created_at: DateTime<Utc>,
    pub author_id: NonZeroU64,
    pub name: String,
    pub short_description: String,
    pub long_description: String,
    pub is_published: bool,
    pub is_official: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub created_at: DateTime<Utc>,
    pub kind: VersionKind,
    pub number: VersionNumber,
    pub data: VersionData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionData {
    pub meta: VersionMeta,
    pub sources: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionKind {
    Stable,
    PreRelease,
    Development,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionNumber {
    pub major: u16,
    pub minor: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMeta {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildPluginSubscription {
    pub guild_id: Id<GuildMarker>,
    pub plugin_id: u64,
    pub version: VersionSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionSelector {
    LatestStable,
    LatestDevel,
    Pinned(VersionNumber),
}

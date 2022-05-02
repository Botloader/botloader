use std::{collections::HashMap, num::NonZeroU64};

use chrono::{DateTime, Utc};
use twilight_model::id::{marker::GuildMarker, Id};

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

pub struct Version {
    pub created_at: DateTime<Utc>,
    pub kind: VersionKind,
    pub number: VersionNumber,
    pub data: VersionData,
}

pub struct VersionData {
    pub meta: VersionMeta,
    pub sources: HashMap<String, String>,
}

pub enum VersionKind {
    Stable,
    PreRelease,
    Development,
}

pub struct VersionNumber {
    pub major: u16,
    pub minor: u16,
}

pub struct VersionMeta {}

pub struct GuildPluginSubscription {
    pub guild_id: Id<GuildMarker>,
    pub plugin_id: u64,
    pub version: VersionSelector,
}

pub enum VersionSelector {
    LatestStable,
    LatestDevel,
    Pinned(VersionNumber),
}

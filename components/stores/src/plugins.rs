use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use twilight_model::id::{
    marker::{GuildMarker, UserMarker},
    Id,
};

#[derive(Debug, Error)]
pub enum PluginStoreError {
    #[error("plugin not found")]
    PluginNotFound,
    #[error("plugin version not found")]
    PluginVersionNotFound,
    #[error("plugin version not found")]
    GuildPluginSubscriptionNotFound,

    #[error("inner error occured: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

#[async_trait]
pub trait PluginStore: Send + Sync {
    async fn list_published_plugins(&self) -> Result<Vec<Plugin>, PluginStoreError>;

    async fn get_plugin(&self, id: u64) -> Result<Plugin, PluginStoreError>;
    async fn search_plugins(&self, query: String) -> Result<Vec<Plugin>, PluginStoreError>;
    async fn create_plugin(&self, plugin: CreatePluginFields) -> Result<Plugin, PluginStoreError>;
    async fn update_plugin(
        &self,
        id: u64,
        plugin: UpdatePluginFields,
    ) -> Result<Plugin, PluginStoreError>;
    async fn delete_plugin(&self, id: u64) -> Result<Plugin, PluginStoreError>;

    async fn get_plugin_version(
        &self,
        plugin_id: u64,
        version: VersionSelector,
    ) -> Result<Version, PluginStoreError>;
    async fn list_plugin_versions(
        &self,
        plugin_id: u64,
    ) -> Result<Vec<VersionMeta>, PluginStoreError>;
    async fn create_plugin_version(
        &self,
        plugin_id: u64,
        fields: CreateVersionFields,
    ) -> Result<Version, PluginStoreError>;
    async fn delete_plugin_version(
        &self,
        plugin_id: u64,
        major: u16,
        minor: u16,
    ) -> Result<Version, PluginStoreError>;

    async fn create_update_guild_plugin_subscription(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: u64,
        version: VersionSelector,
    ) -> Result<GuildPluginSubscription, PluginStoreError>;
    async fn list_guild_plugin_subscriptions(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Vec<GuildPluginSubscription>, PluginStoreError>;
    async fn delete_guild_plugin_subscription(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: u64,
    ) -> Result<GuildPluginSubscription, PluginStoreError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: u64,
    pub created_at: DateTime<Utc>,
    pub author_id: Id<UserMarker>,
    pub name: String,
    pub short_description: String,
    pub long_description: String,
    pub is_published: bool,
    pub is_official: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePluginFields {
    pub author_id: Id<UserMarker>,
    pub name: String,
    pub short_description: String,
    pub long_description: String,
    pub is_published: bool,
    pub is_official: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePluginFields {
    pub author_id: Option<Id<UserMarker>>,
    pub name: Option<String>,
    pub short_description: Option<String>,
    pub long_description: Option<String>,
    pub is_published: Option<bool>,
    pub is_official: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMeta {
    pub plugin_id: u64,
    pub created_at: DateTime<Utc>,
    pub kind: VersionKind,
    pub number: VersionNumber,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub meta: VersionMeta,
    pub data: VersionData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionData {
    pub sources: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVersionFields {
    pub kind: VersionKind,
    pub number: VersionNumber,
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

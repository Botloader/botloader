use std::num::NonZeroU64;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{
    plugin::{Image, Plugin, PluginImageKind, PluginType},
    user::UserMeta,
};
use runtime_models::internal::script::{SettingsOptionDefinition, SettingsOptionValue};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, UserMarker},
    Id,
};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ConfigStoreError {
    #[error("script not found")]
    ScriptNotFound,

    #[error("script link not found")]
    LinkNotFound,

    #[error("reached limit of guild scripts: {0} (limit {1})")]
    GuildScriptLimitReached(u64, u64),

    #[error("inner error occured: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),

    #[error("plugin not found: {0}")]
    PluginNotFound(u64),

    #[error("plugin is already on guild")]
    GuildAlreadyHasPlugin,

    #[error("image not found: {0}/{1}")]
    ImageNotFound(u64, Uuid),
}

impl ConfigStoreError {
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            Self::ScriptNotFound
                | Self::LinkNotFound
                | Self::PluginNotFound(_)
                | Self::ImageNotFound(_, _)
        )
    }
}

pub type ConfigStoreResult<T> = Result<T, ConfigStoreError>;

#[async_trait]
pub trait ConfigStore: Send + Sync {
    async fn get_script(
        &self,
        guild_id: Id<GuildMarker>,
        script_name: String,
    ) -> ConfigStoreResult<Script>;
    async fn get_script_by_id(
        &self,
        guild_id: Id<GuildMarker>,
        script_id: u64,
    ) -> ConfigStoreResult<Script>;
    async fn create_script(
        &self,
        guild_id: Id<GuildMarker>,
        script: CreateScript,
    ) -> ConfigStoreResult<Script>;
    async fn update_script(
        &self,
        guild_id: Id<GuildMarker>,
        script: UpdateScript,
    ) -> ConfigStoreResult<Script>;
    async fn update_script_contributes(
        &self,
        guild_id: Id<GuildMarker>,
        script_id: u64,
        contribs: ScriptContributes,
    ) -> ConfigStoreResult<Script>;
    async fn del_script(
        &self,
        guild_id: Id<GuildMarker>,
        script_name: String,
    ) -> ConfigStoreResult<()>;
    async fn list_scripts(&self, guild_id: Id<GuildMarker>) -> ConfigStoreResult<Vec<Script>>;

    async fn get_guild_meta_config(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> ConfigStoreResult<Option<GuildMetaConfig>>;
    async fn update_guild_meta_config(
        &self,
        conf: &GuildMetaConfig,
    ) -> ConfigStoreResult<GuildMetaConfig>;

    async fn get_guild_meta_config_or_default(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> ConfigStoreResult<GuildMetaConfig> {
        match self.get_guild_meta_config(guild_id).await {
            Ok(Some(conf)) => Ok(conf),
            Ok(None) => Ok(GuildMetaConfig::guild_default(guild_id)),
            Err(e) => Err(e),
        }
    }

    async fn add_update_joined_guild(&self, guild: JoinedGuild) -> ConfigStoreResult<JoinedGuild>;
    async fn set_guild_left_status(
        &self,
        guild_id: Id<GuildMarker>,
        left: bool,
    ) -> ConfigStoreResult<JoinedGuild>;

    async fn get_joined_guilds(
        &self,
        ids: &[Id<GuildMarker>],
    ) -> ConfigStoreResult<Vec<JoinedGuild>>;
    async fn get_joined_guilds_not_in(
        &self,
        ids: &[Id<GuildMarker>],
    ) -> ConfigStoreResult<Vec<JoinedGuild>>;

    async fn is_guild_whitelisted(&self, id: Id<GuildMarker>) -> ConfigStoreResult<bool>;

    async fn delete_guild_config_data(&self, id: Id<GuildMarker>) -> ConfigStoreResult<()>;

    async fn get_left_guilds(&self, threshold_hours: u64) -> ConfigStoreResult<Vec<JoinedGuild>>;

    async fn get_guild_premium_slots(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> ConfigStoreResult<Vec<PremiumSlot>>;

    async fn get_user_premium_slots(
        &self,
        user_id: Id<UserMarker>,
    ) -> ConfigStoreResult<Vec<PremiumSlot>>;

    async fn create_update_premium_slot_by_source(
        &self,
        slot: CreateUpdatePremiumSlotBySource,
    ) -> ConfigStoreResult<PremiumSlot>;

    async fn update_premium_slot_attachment(
        &self,
        user_id: Id<UserMarker>,
        slot_id: u64,
        guild_id: Option<Id<GuildMarker>>,
    ) -> ConfigStoreResult<PremiumSlot>;

    async fn create_plugin(&self, create_plugin: CreatePlugin) -> ConfigStoreResult<Plugin>;
    async fn get_plugin(&self, plugin_id: u64) -> ConfigStoreResult<Plugin>;
    async fn get_plugin_image(&self, plugin_id: u64, image_id: Uuid) -> ConfigStoreResult<Image>;
    async fn get_plugins(&self, plugin_ids: &[u64]) -> ConfigStoreResult<Vec<Plugin>>;
    async fn get_user_plugins(&self, user_id: u64) -> ConfigStoreResult<Vec<Plugin>>;

    async fn get_published_public_plugins(&self) -> ConfigStoreResult<Vec<Plugin>>;
    async fn update_plugin_meta(
        &self,
        plugin_id: u64,
        update_plugin: UpdatePluginMeta,
    ) -> ConfigStoreResult<Plugin>;
    async fn upsert_plugin_image(
        &self,
        plugin_id: u64,
        image: CreateUpdatePluginImage,
    ) -> ConfigStoreResult<()>;
    async fn delete_plugin_image(&self, plugin_id: u64, image_id: Uuid) -> ConfigStoreResult<()>;
    async fn update_script_plugin_dev_version(
        &self,
        plugin_id: u64,
        new_source: String,
    ) -> ConfigStoreResult<Plugin>;
    async fn publish_script_plugin_version(
        &self,
        plugin_id: u64,
        new_source: String,
    ) -> ConfigStoreResult<Vec<Id<GuildMarker>>>;

    async fn try_guild_add_script_plugin(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: u64,
        auto_update: bool,
    ) -> ConfigStoreResult<Script>;

    async fn get_user_meta(&self, user_id: u64) -> ConfigStoreResult<UserMeta>;

    async fn create_image(&self, create: CreateImage) -> ConfigStoreResult<Uuid>;
    async fn soft_delete_image(&self, id: Uuid) -> ConfigStoreResult<Uuid>;
}

/// Struct you get back from the store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: u64,
    pub name: String,
    pub original_source: String,
    pub enabled: bool,
    pub contributes: ScriptContributes,
    pub plugin_id: Option<u64>,
    pub plugin_auto_update: Option<bool>,
    pub plugin_version_number: Option<u32>,
    pub settings_definitions: Option<Vec<SettingsOptionDefinition>>,
    pub settings_values: Vec<SettingsOptionValue>,
}

/// Struct you get back from the store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScript {
    pub id: u64,
    pub name: Option<String>,
    pub original_source: Option<String>,
    pub enabled: Option<bool>,
    pub contributes: Option<ScriptContributes>,
    pub plugin_version_number: Option<u32>,
    pub settings_definitions: Option<Vec<SettingsOptionDefinition>>,
    pub settings_values: Option<Vec<SettingsOptionValue>>,
}

/// Struct used when creating a script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScript {
    pub name: String,
    pub original_source: String,
    pub enabled: bool,
    pub plugin_id: Option<u64>,
    pub plugin_auto_update: Option<bool>,
    pub plugin_version_number: Option<u32>,
}

/// Contribution points for a scripts, e.g triggers, commands etc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptContributes {
    pub commands: Vec<twilight_model::application::command::Command>,
    pub interval_timers: Vec<IntervalTimerContrib>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntervalTimerContrib {
    pub name: String,
    pub interval: crate::timers::IntervalType,
    pub plugin_id: Option<u64>,
}

/// A guilds config, for storing core botloader settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuildMetaConfig {
    pub guild_id: Id<GuildMarker>,
    pub error_channel_id: Option<Id<ChannelMarker>>,
}

impl GuildMetaConfig {
    pub fn guild_default(guild_id: Id<GuildMarker>) -> Self {
        Self {
            guild_id,
            error_channel_id: None,
        }
    }
}

/// A joined guild, we we store all guidls were connected to in the store
#[derive(Debug, Serialize, Deserialize)]
pub struct JoinedGuild {
    pub id: Id<GuildMarker>,
    pub name: String,
    pub icon: String,
    pub owner_id: Id<UserMarker>,
    pub left_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PremiumSlot {
    pub id: u64,
    pub title: String,
    pub user_id: Option<Id<UserMarker>>,
    pub message: String,
    pub source: String,
    pub source_id: String,
    pub tier: PremiumSlotTier,
    pub state: PremiumSlotState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub manage_url: String,
    pub attached_guild_id: Option<Id<GuildMarker>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUpdatePremiumSlotBySource {
    pub title: String,
    pub user_id: Option<Id<UserMarker>>,
    pub message: String,
    pub source: String,
    pub source_id: String,
    pub tier: PremiumSlotTier,
    pub state: PremiumSlotState,
    pub expires_at: DateTime<Utc>,
    pub manage_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PremiumSlotState {
    Active,
    Cancelling,
    Cancelled,
    PaymentFailed,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum PremiumSlotTier {
    Lite,
    Premium,
}

impl PremiumSlotTier {
    pub fn is_higher_than(&self, other: PremiumSlotTier) -> bool {
        matches!(
            (self, other),
            (PremiumSlotTier::Premium, PremiumSlotTier::Lite)
        )
    }
}

pub struct User {
    pub discord_id: NonZeroU64,
    pub username: String,
    pub discriminator: u16,
    pub avatar: String,

    pub is_developer: bool,
    pub is_subscriber: bool,
}

pub struct CreatePlugin {
    pub name: String,
    pub short_description: String,
    pub long_description: String,
    pub is_official: bool,
    pub is_public: bool,
    pub author_id: u64,
    pub kind: PluginType,
}

pub struct CreateUpdatePluginImage {
    pub image_id: Uuid,
    pub description: String,
    pub position: i32,
    pub kind: PluginImageKind,
}

pub struct UpdatePluginMeta {
    pub name: Option<String>,
    pub short_description: Option<String>,
    pub long_description: Option<String>,
    pub is_official: Option<bool>,
    pub author_id: Option<u64>,
    pub is_public: Option<bool>,
    pub is_published: Option<bool>,
}

pub struct CreateImage {
    pub user_id: u64,
    pub plugin_id: u64,
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

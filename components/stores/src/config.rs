use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, UserMarker},
    Id,
};

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
}

/// Struct you get back from the store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: u64,
    pub name: String,
    pub original_source: String,
    pub enabled: bool,
    pub contributes: ScriptContributes,
}

/// Struct you get back from the store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScript {
    pub id: u64,
    pub name: String,
    pub original_source: String,
    pub enabled: bool,
    pub contributes: Option<ScriptContributes>,
}

/// Struct used when creating a script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScript {
    pub name: String,
    pub original_source: String,
    pub enabled: bool,
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

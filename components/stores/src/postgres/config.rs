use super::Postgres;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use twilight_model::id::{ChannelId, GuildId, UserId};

use crate::config::{
    ConfigStoreError, ConfigStoreResult, CreateScript, GuildMetaConfig, JoinedGuild, Script,
    ScriptContributes, UpdateScript,
};

const GUILD_SCRIPT_COUNT_LIMIT: i64 = 100;

impl Postgres {
    async fn get_db_script_by_name(
        &self,
        guild_id: GuildId,
        script_name: &str,
    ) -> ConfigStoreResult<DbScript> {
        match sqlx::query_as!(
            DbScript,
            "SELECT id, guild_id, original_source, name, enabled, contributes_commands, \
             contributes_interval_timers FROM guild_scripts WHERE guild_id = $1 AND name = $2;",
            guild_id.get() as i64,
            script_name
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(s) => Ok(s),
            Err(sqlx::Error::RowNotFound) => Err(ConfigStoreError::ScriptNotFound),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_db_script_by_id(&self, guild_id: GuildId, id: i64) -> ConfigStoreResult<DbScript> {
        Ok(sqlx::query_as!(
            DbScript,
            "SELECT id, guild_id, name, original_source, enabled, contributes_commands, \
             contributes_interval_timers FROM guild_scripts WHERE guild_id = $1 AND id = $2;",
            guild_id.0.get() as i64,
            id
        )
        .fetch_one(&self.pool)
        .await?)
    }

    async fn get_guild_script_count(&self, guild_id: GuildId) -> ConfigStoreResult<i64> {
        let result = sqlx::query!(
            "SELECT count(*) FROM guild_scripts WHERE guild_id = $1;",
            guild_id.0.get() as i64,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or_default())
    }
}

#[async_trait]
impl crate::config::ConfigStore for Postgres {
    async fn get_script(
        &self,
        guild_id: GuildId,
        script_name: String,
    ) -> ConfigStoreResult<Script> {
        Ok(self
            .get_db_script_by_name(guild_id, &script_name)
            .await?
            .into())
    }

    async fn get_script_by_id(
        &self,
        guild_id: GuildId,
        script_id: u64,
    ) -> ConfigStoreResult<Script> {
        Ok(self
            .get_db_script_by_id(guild_id, script_id as i64)
            .await?
            .into())
    }

    async fn create_script(
        &self,
        guild_id: GuildId,
        script: CreateScript,
    ) -> ConfigStoreResult<Script> {
        let count = self.get_guild_script_count(guild_id).await?;
        if count > GUILD_SCRIPT_COUNT_LIMIT {
            return Err(ConfigStoreError::GuildScriptLimitReached(
                count as u64,
                GUILD_SCRIPT_COUNT_LIMIT as u64,
            ));
        }

        let res = sqlx::query_as!(
            DbScript,
            "
                INSERT INTO guild_scripts (guild_id, name, original_source, enabled) 
                VALUES ($1, $2, $3, $4)
                RETURNING id, guild_id, name, original_source, enabled, contributes_commands, \
             contributes_interval_timers;
            ",
            guild_id.0.get() as i64,
            script.name,
            script.original_source,
            script.enabled,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.into())
    }

    async fn update_script(
        &self,
        guild_id: GuildId,
        script: UpdateScript,
    ) -> ConfigStoreResult<Script> {
        let res = if let Some(contribs) = script.contributes {
            let commands_enc = serde_json::to_value(contribs).unwrap();

            sqlx::query_as!(
                DbScript,
                "
                    UPDATE guild_scripts SET
                    original_source = $3,
                    enabled = $4,
                    contributes_commands = $5
                    WHERE guild_id = $1 AND id=$2
                    RETURNING id, name, original_source, guild_id, enabled, contributes_commands, \
                 contributes_interval_timers;
                ",
                guild_id.0.get() as i64,
                script.id as i64,
                script.original_source,
                script.enabled,
                commands_enc,
            )
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                DbScript,
                "
                    UPDATE guild_scripts SET
                    original_source = $3,
                    enabled = $4
                    WHERE guild_id = $1 AND id=$2
                    RETURNING id, name, original_source, guild_id, enabled, contributes_commands, \
                 contributes_interval_timers;
                ",
                guild_id.0.get() as i64,
                script.id as i64,
                script.original_source,
                script.enabled,
            )
            .fetch_one(&self.pool)
            .await?
        };

        Ok(res.into())
    }

    async fn update_script_contributes(
        &self,
        guild_id: GuildId,
        script_id: u64,
        contribs: ScriptContributes,
    ) -> ConfigStoreResult<Script> {
        let commands_enc = serde_json::to_value(contribs.commands).unwrap();
        let intervals_enc = serde_json::to_value(contribs.interval_timers).unwrap();

        let res = sqlx::query_as!(
            DbScript,
            "
                    UPDATE guild_scripts SET
                    contributes_commands = $3,
                    contributes_interval_timers = $4
                    WHERE guild_id = $1 AND id=$2
                    RETURNING id, name, original_source, guild_id, enabled, contributes_commands, \
             contributes_interval_timers;
                ",
            guild_id.0.get() as i64,
            script_id as i64,
            commands_enc,
            intervals_enc,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.into())
    }

    async fn del_script(&self, guild_id: GuildId, script_name: String) -> ConfigStoreResult<()> {
        let res = sqlx::query!(
            "DELETE FROM guild_scripts WHERE guild_id = $1 AND name = $2;",
            guild_id.0.get() as i64,
            script_name
        )
        .execute(&self.pool)
        .await?;

        if res.rows_affected() > 0 {
            Ok(())
        } else {
            Err(ConfigStoreError::ScriptNotFound)
        }
    }

    async fn list_scripts(&self, guild_id: GuildId) -> ConfigStoreResult<Vec<Script>> {
        let res = sqlx::query_as!(
            DbScript,
            "SELECT id, guild_id, original_source, name, enabled, contributes_commands, \
             contributes_interval_timers FROM guild_scripts WHERE guild_id = $1",
            guild_id.0.get() as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(res.into_iter().map(|e| e.into()).collect())
    }

    async fn get_guild_meta_config(
        &self,
        guild_id: GuildId,
    ) -> ConfigStoreResult<Option<GuildMetaConfig>> {
        match sqlx::query_as!(
            DbGuildMetaConfig,
            "SELECT guild_id, error_channel_id FROM guild_meta_configs
        WHERE guild_id = $1;",
            guild_id.0.get() as i64,
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(conf) => Ok(Some(conf.into())),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn update_guild_meta_config(
        &self,
        conf: &GuildMetaConfig,
    ) -> ConfigStoreResult<GuildMetaConfig> {
        let db_conf = sqlx::query_as!(
            DbGuildMetaConfig,
            "INSERT INTO guild_meta_configs (guild_id, error_channel_id) VALUES ($1, $2)
            ON CONFLICT (guild_id) DO UPDATE SET
            error_channel_id = $2
            RETURNING guild_id, error_channel_id;",
            conf.guild_id.0.get() as i64,
            conf.error_channel_id
                .map(|e| e.0.get() as i64)
                .unwrap_or_default(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(db_conf.into())
    }

    async fn add_update_joined_guild(&self, guild: JoinedGuild) -> ConfigStoreResult<JoinedGuild> {
        let db_guild = sqlx::query_as!(
            DbJoinedGuild,
            "INSERT INTO joined_guilds (id, name, icon, owner_id, left_at) VALUES ($1, $2, $3, \
             $4, null)
            ON CONFLICT (id) DO UPDATE SET 
            name = $2, icon = $3, owner_id = $4, left_at = null
            RETURNING id, name, icon, owner_id, left_at;",
            guild.id.0.get() as i64,
            &guild.name,
            &guild.icon,
            guild.owner_id.0.get() as i64,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(db_guild.into())
    }

    async fn set_guild_left_status(
        &self,
        guild_id: GuildId,
        left: bool,
    ) -> ConfigStoreResult<JoinedGuild> {
        let db_guild = sqlx::query_as!(
            DbJoinedGuild,
            "UPDATE joined_guilds SET left_at = CASE 
                WHEN left_at IS NULL AND $2 = true THEN now()
                WHEN $2 = false THEN null
                ELSE left_at
                END
            WHERE id = $1 RETURNING id, name, icon, owner_id, left_at;",
            guild_id.0.get() as i64,
            left
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(db_guild.into())
    }

    async fn get_joined_guilds(&self, ids: &[GuildId]) -> ConfigStoreResult<Vec<JoinedGuild>> {
        let guilds = sqlx::query_as!(
            DbJoinedGuild,
            "SELECT id, name, icon, owner_id, left_at FROM joined_guilds WHERE id = ANY ($1)",
            &ids.into_iter()
                .map(|e| e.0.get() as i64)
                .collect::<Vec<_>>(),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(guilds.into_iter().map(|e| e.into()).collect())
    }

    async fn is_guild_whitelisted(&self, guild_id: GuildId) -> ConfigStoreResult<bool> {
        let result = sqlx::query!(
            "SELECT count(*) FROM guild_whitelist WHERE guild_id = $1;",
            guild_id.get() as i64,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or_default() > 0)
    }
}

#[allow(dead_code)]
struct DbScript {
    id: i64,
    guild_id: i64,
    original_source: String,
    name: String,
    enabled: bool,
    contributes_commands: serde_json::Value,
    contributes_interval_timers: serde_json::Value,
}

impl From<DbScript> for Script {
    fn from(script: DbScript) -> Self {
        let commands_dec = serde_json::from_value(script.contributes_commands).unwrap_or_default();
        let intervals_dec =
            serde_json::from_value(script.contributes_interval_timers).unwrap_or_default();

        Self {
            id: script.id as u64,
            name: script.name,
            original_source: script.original_source,
            enabled: script.enabled,
            contributes: ScriptContributes {
                commands: commands_dec,
                interval_timers: intervals_dec,
            },
        }
    }
}

struct DbGuildMetaConfig {
    pub guild_id: i64,
    pub error_channel_id: i64,
}

impl From<DbGuildMetaConfig> for GuildMetaConfig {
    fn from(mc: DbGuildMetaConfig) -> Self {
        Self {
            guild_id: GuildId::new(mc.guild_id as u64).unwrap(),
            error_channel_id: if mc.error_channel_id != 0 {
                Some(ChannelId::new(mc.error_channel_id as u64).unwrap())
            } else {
                None
            },
        }
    }
}

pub struct DbJoinedGuild {
    pub id: i64,
    pub name: String,
    pub icon: String,
    pub owner_id: i64,
    pub left_at: Option<DateTime<Utc>>,
}

impl From<DbJoinedGuild> for JoinedGuild {
    fn from(g: DbJoinedGuild) -> Self {
        Self {
            id: GuildId::new(g.id as u64).unwrap(),
            name: g.name,
            icon: g.icon,
            owner_id: UserId::new(g.owner_id as u64).unwrap(),
            left_at: g.left_at,
        }
    }
}

impl From<sqlx::Error> for ConfigStoreError {
    fn from(err: sqlx::Error) -> Self {
        Self::Other(Box::new(err))
    }
}

use super::Postgres;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{
    plugin::{Image, Plugin, PluginData, PluginImage, PluginImageKind, ScriptPluginData},
    user::UserMeta,
};
use sqlx::{postgres::types::PgInterval, PgConnection};
use twilight_model::id::{
    marker::{GuildMarker, UserMarker},
    Id,
};
use uuid::Uuid;

use crate::config::{
    ConfigStoreError, ConfigStoreResult, CreateImage, CreatePlugin, CreateScript,
    CreateUpdatePluginImage, CreateUpdatePremiumSlotBySource, GuildMetaConfig, JoinedGuild,
    PremiumSlot, PremiumSlotState, PremiumSlotTier, Script, ScriptContributes, UpdatePluginMeta,
    UpdateScript,
};

const GUILD_SCRIPT_COUNT_LIMIT: i64 = 100;

impl Postgres {
    async fn get_db_script_by_name(
        &self,
        guild_id: Id<GuildMarker>,
        script_name: &str,
    ) -> ConfigStoreResult<DbScript> {
        match sqlx::query_as!(
            DbScript,
            "SELECT id, guild_id, original_source, name, enabled, contributes_commands, \
             contributes_interval_timers, plugin_id, plugin_auto_update, plugin_version_number, \
             settings_definitions, settings_values FROM guild_scripts WHERE guild_id = $1 AND \
             name = $2 AND plugin_id IS NULL;",
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

    async fn get_db_script_by_id(
        &self,
        guild_id: Id<GuildMarker>,
        id: i64,
    ) -> ConfigStoreResult<DbScript> {
        Ok(sqlx::query_as!(
            DbScript,
            "SELECT id, guild_id, name, original_source, enabled, contributes_commands, \
             contributes_interval_timers, plugin_id, plugin_auto_update, plugin_version_number, \
             settings_definitions, settings_values FROM guild_scripts WHERE guild_id = $1 AND id \
             = $2;",
            guild_id.get() as i64,
            id
        )
        .fetch_one(&self.pool)
        .await?)
    }

    async fn get_guild_script_count(
        conn: &mut PgConnection,
        guild_id: Id<GuildMarker>,
    ) -> ConfigStoreResult<i64> {
        let result = sqlx::query!(
            "SELECT count(*) FROM guild_scripts WHERE guild_id = $1;",
            guild_id.get() as i64,
        )
        .fetch_one(conn)
        .await?;

        Ok(result.count.unwrap_or_default())
    }

    async fn get_guild_scripts(
        conn: &mut PgConnection,
        guild_id: Id<GuildMarker>,
    ) -> ConfigStoreResult<Vec<Script>> {
        let res = sqlx::query_as!(
            DbScript,
            "SELECT id, guild_id, original_source, name, enabled, contributes_commands, \
             contributes_interval_timers, plugin_id, plugin_auto_update, plugin_version_number, \
             settings_definitions, settings_values FROM guild_scripts WHERE guild_id = $1 ORDER \
             BY id ASC",
            guild_id.get() as i64,
        )
        .fetch_all(conn)
        .await?;

        Ok(res.into_iter().map(|e| e.into()).collect())
    }

    async fn inner_create_script(
        conn: &mut PgConnection,
        guild_id: Id<GuildMarker>,
        script: CreateScript,
    ) -> ConfigStoreResult<Script> {
        let count = Self::get_guild_script_count(conn, guild_id).await?;
        if count > GUILD_SCRIPT_COUNT_LIMIT {
            return Err(ConfigStoreError::GuildScriptLimitReached(
                count as u64,
                GUILD_SCRIPT_COUNT_LIMIT as u64,
            ));
        }

        let res = sqlx::query_as!(
            DbScript,
            "INSERT INTO guild_scripts (guild_id, name, original_source, enabled, plugin_id, \
             plugin_auto_update, plugin_version_number) 
VALUES ($1, $2, $3, $4, $5, $6, $7)
RETURNING id, guild_id, name, original_source, enabled, contributes_commands, \
             contributes_interval_timers, plugin_id, plugin_auto_update, plugin_version_number, \
             settings_definitions, settings_values;",
            guild_id.get() as i64,
            script.name,
            script.original_source,
            script.enabled,
            script.plugin_id.map(|v| v as i64),
            script.plugin_auto_update,
            script.plugin_version_number.map(|v| v as i32),
        )
        .fetch_one(conn)
        .await?;

        Ok(res.into())
    }

    async fn get_plugin_images_with_pool(
        &self,
        plugin_id: u64,
    ) -> ConfigStoreResult<Vec<DbPluginImage>> {
        Self::get_plugin_images_with_conn(&mut *self.pool.acquire().await?, plugin_id).await
    }

    async fn get_plugin_images_with_conn(
        conn: &mut PgConnection,
        plugin_id: u64,
    ) -> ConfigStoreResult<Vec<DbPluginImage>> {
        Ok(sqlx::query_as!(
            DbPluginImage,
            "SELECT plugin_images.*, width, height FROM plugin_images INNER JOIN images ON \
             images.id = plugin_images.image_id WHERE plugin_images.plugin_id = $1 ORDER BY \
             position DESC",
            plugin_id as i64
        )
        .fetch_all(conn)
        .await?)
    }

    async fn inner_get_plugin(
        conn: &mut PgConnection,
        plugin_id: u64,
    ) -> ConfigStoreResult<Plugin> {
        let db_plugin = sqlx::query_as!(
            DbPlugin,
            r#"SELECT id,
created_at,
name,
short_description,
long_description,
is_published,
is_official,
plugin_kind,
current_version_number,
script_published_source,
script_published_version_updated_at,
script_dev_source,
script_dev_version_updated_at,
author_id,
is_public
FROM plugins WHERE id = $1"#,
            plugin_id as i64,
        )
        .fetch_optional(&mut *conn)
        .await?
        .ok_or(ConfigStoreError::PluginNotFound(plugin_id))?;

        let images = Self::get_plugin_images_with_conn(conn, plugin_id).await?;

        Ok(PluginAndImages(db_plugin, images).into())
    }

    async fn inner_upsert_plugin_image(
        &self,
        plugin_id: u64,
        image: CreateUpdatePluginImage,
    ) -> ConfigStoreResult<()> {
        sqlx::query!(
            "INSERT INTO plugin_images (plugin_id, image_id, created_at, description, position, \
             kind)
            VALUES ($1, $2, now(), $3, $4, $5)
            ON CONFLICT (plugin_id, image_id) DO UPDATE SET
            description = $3;",
            plugin_id as i64,
            image.image_id,
            image.description,
            image.position,
            i32::from(image.kind),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl crate::config::ConfigStore for Postgres {
    async fn get_script(
        &self,
        guild_id: Id<GuildMarker>,
        script_name: String,
    ) -> ConfigStoreResult<Script> {
        Ok(self
            .get_db_script_by_name(guild_id, &script_name)
            .await?
            .into())
    }

    async fn get_script_by_id(
        &self,
        guild_id: Id<GuildMarker>,
        script_id: u64,
    ) -> ConfigStoreResult<Script> {
        Ok(self
            .get_db_script_by_id(guild_id, script_id as i64)
            .await?
            .into())
    }

    async fn create_script(
        &self,
        guild_id: Id<GuildMarker>,
        script: CreateScript,
    ) -> ConfigStoreResult<Script> {
        Self::inner_create_script(&mut *self.pool.acquire().await?, guild_id, script).await
    }

    async fn update_script(
        &self,
        guild_id: Id<GuildMarker>,
        script: UpdateScript,
    ) -> ConfigStoreResult<Script> {
        let commands_enc = script.contributes.map(|v| serde_json::to_value(v).unwrap());
        let settings_definitions = script
            .settings_definitions
            .map(|v| serde_json::to_value(v).unwrap());
        let settings_values = script
            .settings_values
            .map(|v| serde_json::to_value(v).unwrap());

        let res = sqlx::query_as!(
            DbScript,
            "
                    UPDATE guild_scripts SET
                    original_source = COALESCE($3, guild_scripts.original_source),
                    enabled = COALESCE($4, guild_scripts.enabled),
                    contributes_commands = COALESCE($5, guild_scripts.contributes_commands),
                    plugin_version_number = COALESCE($6, guild_scripts.plugin_version_number),
                    settings_definitions = COALESCE($7, guild_scripts.settings_definitions),
                    settings_values = COALESCE($8, guild_scripts.settings_values)
                    WHERE guild_id = $1 AND id=$2
                    RETURNING id, name, original_source, guild_id, enabled, contributes_commands, \
             contributes_interval_timers, plugin_id, plugin_auto_update, plugin_version_number, \
             settings_definitions, settings_values;
                ",
            guild_id.get() as i64,
            script.id as i64,
            script.original_source,
            script.enabled,
            commands_enc,
            script.plugin_version_number.map(|v| v as i32),
            settings_definitions,
            settings_values,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.into())
    }

    async fn update_script_contributes(
        &self,
        guild_id: Id<GuildMarker>,
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
             contributes_interval_timers, plugin_id, plugin_auto_update, plugin_version_number, \
             settings_definitions, settings_values;
                ",
            guild_id.get() as i64,
            script_id as i64,
            commands_enc,
            intervals_enc,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.into())
    }

    async fn del_script(
        &self,
        guild_id: Id<GuildMarker>,
        script_name: String,
    ) -> ConfigStoreResult<()> {
        let res = sqlx::query!(
            "DELETE FROM guild_scripts WHERE guild_id = $1 AND name = $2;",
            guild_id.get() as i64,
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

    async fn list_scripts(&self, guild_id: Id<GuildMarker>) -> ConfigStoreResult<Vec<Script>> {
        Self::get_guild_scripts(&mut *self.pool.acquire().await?, guild_id).await
    }

    async fn get_guild_meta_config(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> ConfigStoreResult<Option<GuildMetaConfig>> {
        match sqlx::query_as!(
            DbGuildMetaConfig,
            "SELECT guild_id, error_channel_id FROM guild_meta_configs
        WHERE guild_id = $1;",
            guild_id.get() as i64,
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
            conf.guild_id.get() as i64,
            conf.error_channel_id
                .map(|e| e.get() as i64)
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
            guild.id.get() as i64,
            &guild.name,
            &guild.icon,
            guild.owner_id.get() as i64,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(db_guild.into())
    }

    async fn set_guild_left_status(
        &self,
        guild_id: Id<GuildMarker>,
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
            guild_id.get() as i64,
            left
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(db_guild.into())
    }

    async fn get_joined_guilds(
        &self,
        ids: &[Id<GuildMarker>],
    ) -> ConfigStoreResult<Vec<JoinedGuild>> {
        let guilds = sqlx::query_as!(
            DbJoinedGuild,
            "SELECT id, name, icon, owner_id, left_at FROM joined_guilds WHERE id = ANY ($1) AND \
             left_at IS NULL",
            &ids.into_iter().map(|e| e.get() as i64).collect::<Vec<_>>(),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(guilds.into_iter().map(|e| e.into()).collect())
    }

    async fn get_joined_guilds_not_in(
        &self,
        ids: &[Id<GuildMarker>],
    ) -> ConfigStoreResult<Vec<JoinedGuild>> {
        let guilds = sqlx::query_as!(
            DbJoinedGuild,
            "SELECT id, name, icon, owner_id, left_at FROM joined_guilds WHERE NOT id = ANY ($1) \
             AND left_at IS NULL",
            &ids.into_iter().map(|e| e.get() as i64).collect::<Vec<_>>(),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(guilds.into_iter().map(|e| e.into()).collect())
    }

    async fn get_left_guilds(&self, threshold_hours: u64) -> ConfigStoreResult<Vec<JoinedGuild>> {
        let interval = PgInterval {
            days: 0,
            months: 0,
            microseconds: threshold_hours as i64 * 1000 * 1000 * 60 * 60,
        };

        let guilds = sqlx::query_as!(
            DbJoinedGuild,
            "SELECT id, name, icon, owner_id, left_at FROM joined_guilds WHERE left_at IS NOT \
             NULL AND left_at < (now() - $1::interval);",
            interval,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(guilds.into_iter().map(|e| e.into()).collect())
    }

    async fn is_guild_whitelisted(&self, guild_id: Id<GuildMarker>) -> ConfigStoreResult<bool> {
        let result = sqlx::query!(
            "SELECT count(*) FROM guild_whitelist WHERE guild_id = $1;",
            guild_id.get() as i64,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or_default() > 0)
    }

    async fn delete_guild_config_data(&self, id: Id<GuildMarker>) -> ConfigStoreResult<()> {
        sqlx::query!("DELETE FROM joined_guilds WHERE id = $1;", id.get() as i64)
            .execute(&self.pool)
            .await?;

        // TODO: should we delete guild scripts as well?

        Ok(())
    }

    async fn get_guild_premium_slots(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> ConfigStoreResult<Vec<PremiumSlot>> {
        let res = sqlx::query_as!(
            DbPremiumSlot,
            "SELECT id, title, user_id, message, source, source_id, tier, state, created_at, \
             updated_at, expires_at, manage_url, attached_guild_id
             FROM premium_slots WHERE attached_guild_id = $1 ORDER BY id ASC;",
            guild_id.get() as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(res.into_iter().map(Into::into).collect())
    }

    async fn get_user_premium_slots(
        &self,
        user_id: Id<UserMarker>,
    ) -> ConfigStoreResult<Vec<PremiumSlot>> {
        let res = sqlx::query_as!(
            DbPremiumSlot,
            "SELECT id, title, user_id, message, source, source_id, tier, state, created_at, \
             updated_at, expires_at, manage_url, attached_guild_id
             FROM premium_slots WHERE user_id = $1 ORDER BY id ASC;",
            user_id.get() as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(res.into_iter().map(Into::into).collect())
    }

    async fn create_update_premium_slot_by_source(
        &self,
        slot: CreateUpdatePremiumSlotBySource,
    ) -> ConfigStoreResult<PremiumSlot> {
        let res = sqlx::query_as!(
            DbPremiumSlot,
            r#"
INSERT INTO premium_slots 
       (title, user_id, message, source, source_id, tier, state, created_at, updated_at,
          expires_at, manage_url, attached_guild_id) 
VALUES ($1,       $2,      $3,     $4,       $5,     $6,    $7,     now(),      now(),
            $8,          $9,           null        )
ON CONFLICT (source, source_id) DO UPDATE SET
    title = $1,
    user_id = $2,
    message = $3,
    source = $4,
    source_id = $5,
    tier = $6,
    state = $7,
    updated_at = now(),
    expires_at = $8,
    manage_url = $9
RETURNING id, title, user_id, message, source, source_id, tier, state, created_at, 
            updated_at, expires_at, manage_url, attached_guild_id;
             "#,
            slot.title,
            slot.user_id.map(|v| v.get() as i64),
            slot.message,
            slot.source,
            slot.source_id,
            tier_to_int(slot.tier),
            state_to_int(slot.state),
            slot.expires_at,
            slot.manage_url,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.into())
    }

    async fn update_premium_slot_attachment(
        &self,
        user_id: Id<UserMarker>,
        slot_id: u64,
        guild_id: Option<Id<GuildMarker>>,
    ) -> ConfigStoreResult<PremiumSlot> {
        let res = sqlx::query_as!(
            DbPremiumSlot,
            r#"
UPDATE premium_slots SET attached_guild_id = $3
WHERE id = $1 AND user_id = $2
RETURNING id, title, user_id, message, source, source_id, tier, state, created_at, 
            updated_at, expires_at, manage_url, attached_guild_id;
             "#,
            slot_id as i64,
            user_id.get() as i64,
            guild_id.map(|v| v.get() as i64),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.into())
    }

    async fn create_plugin(&self, create_plugin: CreatePlugin) -> ConfigStoreResult<Plugin> {
        let res = sqlx::query_as!(
            DbPlugin,
            r#"INSERT INTO plugins (
    created_at,
    name,
    short_description,
    long_description,
    is_published,
    is_official,
    plugin_kind,
    current_version_number,
    script_published_source,
    script_published_version_updated_at,
    script_dev_source,
    script_dev_version_updated_at,
    author_id,
    is_public
) VALUES (
    now(), -- created_at
    $1, -- name
    $2, -- short_description
    $3, -- long_description
    false, -- is_published
    $4, -- is_official
    $5, -- plugin_kind
    0, -- current_version_number
    null, -- script_published_source
    null, -- script_published_version_updated_at
    null, -- script_dev_source
    null, -- script_dev_version_updated_at
    $6, -- author_id
    $7 -- is_public
) RETURNING id,
created_at,
name,
short_description,
long_description,
is_published,
is_official,
plugin_kind,
current_version_number,
script_published_source,
script_published_version_updated_at,
script_dev_source,
script_dev_version_updated_at,
author_id,
is_public"#,
            create_plugin.name,
            create_plugin.short_description,
            create_plugin.long_description,
            create_plugin.is_official,
            create_plugin.kind as i16,
            create_plugin.author_id as i64,
            create_plugin.is_public,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(PluginAndImages(res, Vec::new()).into())
    }

    async fn update_plugin_meta(
        &self,
        plugin_id: u64,
        update_plugin: UpdatePluginMeta,
    ) -> ConfigStoreResult<Plugin> {
        let res = sqlx::query_as!(
            DbPlugin,
            r#"UPDATE plugins SET
name = COALESCE($2, plugins.name),
short_description = COALESCE($3, plugins.short_description),
long_description = COALESCE($4, plugins.long_description),
is_official = COALESCE($5, plugins.is_official),
author_id = COALESCE($6, plugins.author_id),
is_public = COALESCE($7, plugins.is_public),
is_published = COALESCE($8, plugins.is_published)
WHERE id = $1
RETURNING id,
created_at,
name,
short_description,
long_description,
is_published,
is_official,
plugin_kind,
current_version_number,
script_published_source,
script_published_version_updated_at,
script_dev_source,
script_dev_version_updated_at,
author_id,
is_public"#,
            plugin_id as i64,
            update_plugin.name,
            update_plugin.short_description,
            update_plugin.long_description,
            update_plugin.is_official,
            update_plugin.author_id.map(|v| v as i64),
            update_plugin.is_public,
            update_plugin.is_published,
        )
        .fetch_one(&self.pool)
        .await?;

        let images = self.get_plugin_images_with_pool(plugin_id).await?;

        Ok(PluginAndImages(res, images).into())
    }

    async fn upsert_plugin_image(
        &self,
        plugin_id: u64,
        image: CreateUpdatePluginImage,
    ) -> ConfigStoreResult<()> {
        if matches!(&image.kind, PluginImageKind::Icon | PluginImageKind::Banner) {
            // There can only be 1 of type icon and banner
            // Possibly delete existing one if it doesn't match
            // TODO: use a transaction here to avoid race conditions
            let res = sqlx::query!(
                "SELECT image_id from plugin_images WHERE plugin_id = $1 AND kind = $2",
                plugin_id as i64,
                i32::from(image.kind)
            )
            .fetch_all(&self.pool)
            .await?;

            for item in res {
                if item.image_id != image.image_id {
                    self.delete_plugin_image(plugin_id, item.image_id).await?;
                }
            }
        }

        self.inner_upsert_plugin_image(plugin_id, image).await?;

        Ok(())
    }

    async fn delete_plugin_image(&self, plugin_id: u64, image_id: Uuid) -> ConfigStoreResult<()> {
        sqlx::query!(
            "DELETE FROM plugin_images WHERE plugin_id = $1 AND image_id = $2",
            plugin_id as i64,
            image_id,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            "UPDATE images SET deleted_at = COALESCE(images.deleted_at, now()) WHERE plugin_id = \
             $1 and id = $2",
            plugin_id as i64,
            image_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_script_plugin_dev_version(
        &self,
        plugin_id: u64,
        new_source: String,
    ) -> ConfigStoreResult<Plugin> {
        let res = sqlx::query_as!(
            DbPlugin,
            r#"UPDATE plugins SET
script_dev_source = $2, 
script_dev_version_updated_at = now()
WHERE id = $1
RETURNING id,
created_at,
name,
short_description,
long_description,
is_published,
is_official,
plugin_kind,
current_version_number,
script_published_source,
script_published_version_updated_at,
script_dev_source,
script_dev_version_updated_at,
author_id,
is_public"#,
            plugin_id as i64,
            new_source,
        )
        .fetch_one(&self.pool)
        .await?;

        let images = self.get_plugin_images_with_pool(plugin_id).await?;

        Ok(PluginAndImages(res, images).into())
    }

    async fn publish_script_plugin_version(
        &self,
        plugin_id: u64,
        new_source: String,
    ) -> ConfigStoreResult<Vec<Id<GuildMarker>>> {
        let plugin = sqlx::query_as!(
            DbPlugin,
            r#"UPDATE plugins SET
script_published_source = $2, 
script_published_version_updated_at = now(),
current_version_number = current_version_number +1
WHERE id = $1
RETURNING id,
created_at,
name,
short_description,
long_description,
is_published,
is_official,
plugin_kind,
current_version_number,
script_published_source,
script_published_version_updated_at,
script_dev_source,
script_dev_version_updated_at,
author_id,
is_public"#,
            plugin_id as i64,
            new_source,
        )
        .fetch_one(&self.pool)
        .await?;

        struct Row {
            guild_id: i64,
        }

        let updated_guilds = sqlx::query_as!(
            Row,
            "UPDATE guild_scripts SET original_source = $2, plugin_version_number = $3 WHERE \
             plugin_id = $1 AND plugin_auto_update RETURNING guild_id",
            plugin_id as i64,
            new_source,
            plugin.current_version_number,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(updated_guilds
            .into_iter()
            .map(|v| Id::new(v.guild_id as u64))
            .collect())
    }

    async fn get_user_meta(&self, user_id: u64) -> ConfigStoreResult<UserMeta> {
        let res = sqlx::query_as!(
            DbUserMeta,
            r#"SELECT discord_user_id, is_admin, is_moderator, is_verified FROM user_meta WHERE discord_user_id = $1"#,
            user_id as i64,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(res.map(Into::into).unwrap_or_default())
    }

    async fn get_plugin(&self, plugin_id: u64) -> ConfigStoreResult<Plugin> {
        Self::inner_get_plugin(&mut *self.pool.acquire().await?, plugin_id).await
    }

    async fn get_plugins(&self, plugin_ids: &[u64]) -> ConfigStoreResult<Vec<Plugin>> {
        let ids = plugin_ids.iter().map(|v| *v as i64).collect::<Vec<_>>();
        let raw_plugins = sqlx::query_as!(
            DbPlugin,
            r#"SELECT id,
created_at,
name,
short_description,
long_description,
is_published,
is_official,
plugin_kind,
current_version_number,
script_published_source,
script_published_version_updated_at,
script_dev_source,
script_dev_version_updated_at,
author_id,
is_public
FROM plugins WHERE id = ANY($1)
ORDER BY id ASC"#,
            &ids,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut res: Vec<Plugin> = Vec::new();
        for plugin in raw_plugins.into_iter() {
            let images = self.get_plugin_images_with_pool(plugin.id as u64).await?;
            res.push(PluginAndImages(plugin, images).into());
        }

        Ok(res)
    }

    async fn get_user_plugins(&self, user_id: u64) -> ConfigStoreResult<Vec<Plugin>> {
        let raw_plugins = sqlx::query_as!(
            DbPlugin,
            r#"SELECT id,
created_at,
name,
short_description,
long_description,
is_published,
is_official,
plugin_kind,
current_version_number,
script_published_source,
script_published_version_updated_at,
script_dev_source,
script_dev_version_updated_at,
author_id,
is_public
FROM plugins WHERE author_id = $1
ORDER BY id ASC"#,
            user_id as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut res: Vec<Plugin> = Vec::new();
        for plugin in raw_plugins.into_iter() {
            let images = self.get_plugin_images_with_pool(plugin.id as u64).await?;
            res.push(PluginAndImages(plugin, images).into());
        }

        Ok(res)
    }

    async fn get_published_public_plugins(&self) -> ConfigStoreResult<Vec<Plugin>> {
        let raw_plugins = sqlx::query_as!(
            DbPlugin,
            r#"SELECT id,
created_at,
name,
short_description,
long_description,
is_published,
is_official,
plugin_kind,
current_version_number,
script_published_source,
script_published_version_updated_at,
script_dev_source,
script_dev_version_updated_at,
author_id,
is_public
FROM plugins WHERE is_published = true AND is_public = true
ORDER BY id ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut res: Vec<Plugin> = Vec::new();
        for plugin in raw_plugins.into_iter() {
            let images = self.get_plugin_images_with_pool(plugin.id as u64).await?;
            res.push(PluginAndImages(plugin, images).into());
        }

        Ok(res)
    }

    async fn try_guild_add_script_plugin(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: u64,
        auto_update: bool,
    ) -> ConfigStoreResult<Script> {
        let mut tx = self.pool.begin().await?;

        let scripts = Self::get_guild_scripts(&mut tx, guild_id).await?;
        if scripts.iter().any(|v| v.plugin_id == Some(plugin_id)) {
            return Err(ConfigStoreError::GuildAlreadyHasPlugin);
        }

        let plugin = Self::inner_get_plugin(&mut tx, plugin_id).await?;

        let source = match plugin.data {
            PluginData::ScriptPlugin(d) => d.published_version.unwrap_or_default(),
        };

        let created = Self::inner_create_script(
            &mut tx,
            guild_id,
            CreateScript {
                name: plugin.name.clone(),
                original_source: source,
                enabled: true,
                plugin_auto_update: Some(auto_update),
                plugin_id: Some(plugin_id),
                plugin_version_number: Some(plugin.current_version),
            },
        )
        .await?;

        tx.commit().await?;

        Ok(created)
    }

    async fn get_plugin_image(&self, plugin_id: u64, image_id: Uuid) -> ConfigStoreResult<Image> {
        let res = sqlx::query!(
            "SELECT * FROM images WHERE plugin_id = $1 AND id = $2",
            plugin_id as i64,
            image_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(res) = res else {
            return Err(ConfigStoreError::ImageNotFound(plugin_id, image_id));
        };

        Ok(Image {
            id: res.id,
            plugin_id: Some(plugin_id),
            uploaded_by: res.uploaded_by as u64,
            width: res.width as u32,
            height: res.height as u32,
            bytes: res.bytes,
            created_at: res.created_at,
            deleted_at: res.deleted_at,
        })
    }

    async fn create_image(&self, create: CreateImage) -> ConfigStoreResult<Uuid> {
        let res = sqlx::query!(
            "INSERT INTO images (uploaded_by, plugin_id, width, height, bytes, created_at)
            VALUES ($1, $2, $3, $4, $5, now())
            RETURNING id;",
            create.user_id as i64,
            create.plugin_id as i64,
            create.width as i32,
            create.height as i32,
            create.bytes,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.id)
    }

    async fn soft_delete_image(&self, _id: Uuid) -> ConfigStoreResult<Uuid> {
        todo!()
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
    plugin_id: Option<i64>,
    plugin_auto_update: Option<bool>,
    plugin_version_number: Option<i32>,
    settings_definitions: serde_json::Value,
    settings_values: serde_json::Value,
}

impl From<DbScript> for Script {
    fn from(script: DbScript) -> Self {
        let commands_dec = serde_json::from_value(script.contributes_commands).unwrap_or_default();
        let intervals_dec =
            serde_json::from_value(script.contributes_interval_timers).unwrap_or_default();

        let settings_definitions =
            serde_json::from_value(script.settings_definitions).unwrap_or_default();
        let settings_values = serde_json::from_value(script.settings_values).unwrap_or_default();

        Self {
            id: script.id as u64,
            name: script.name,
            original_source: script.original_source,
            enabled: script.enabled,
            contributes: ScriptContributes {
                commands: commands_dec,
                interval_timers: intervals_dec,
            },
            plugin_id: script.plugin_id.map(|v| v as u64),
            plugin_auto_update: script.plugin_auto_update,
            plugin_version_number: script.plugin_version_number.map(|v| v as u32),
            settings_definitions,
            settings_values,
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
            guild_id: Id::new(mc.guild_id as u64),
            error_channel_id: if mc.error_channel_id != 0 {
                Some(Id::new(mc.error_channel_id as u64))
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
            id: Id::new(g.id as u64),
            name: g.name,
            icon: g.icon,
            owner_id: Id::new(g.owner_id as u64),
            left_at: g.left_at,
        }
    }
}

impl From<sqlx::Error> for ConfigStoreError {
    fn from(err: sqlx::Error) -> Self {
        Self::Other(Box::new(err))
    }
}

struct DbPremiumSlot {
    id: i64,
    title: String,
    user_id: Option<i64>,
    message: String,
    source: String,
    source_id: String,
    tier: i32,
    state: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    manage_url: String,
    attached_guild_id: Option<i64>,
}

impl From<DbPremiumSlot> for PremiumSlot {
    fn from(v: DbPremiumSlot) -> Self {
        Self {
            id: v.id as u64,
            title: v.title,
            user_id: v.user_id.map(|uid| Id::new(uid as u64)),
            message: v.message,
            source: v.source,
            source_id: v.source_id,
            tier: match v.tier {
                1 => PremiumSlotTier::Lite,
                2 => PremiumSlotTier::Premium,
                _ => panic!("unknown premium slot tier, id: {}, tier: {}", v.id, v.tier),
            },
            state: match v.state {
                1 => PremiumSlotState::Active,
                2 => PremiumSlotState::Cancelling,
                3 => PremiumSlotState::Cancelled,
                4 => PremiumSlotState::PaymentFailed,
                _ => panic!(
                    "unknown premium slot state, id: {}, state: {}",
                    v.id, v.state
                ),
            },
            created_at: v.created_at,
            updated_at: v.updated_at,
            expires_at: v.expires_at,
            manage_url: v.manage_url,
            attached_guild_id: v.attached_guild_id.map(|gid| Id::new(gid as u64)),
        }
    }
}

// since the int representation is specific to this postgres implementation, i don't want
// to implement From<PremiumSlotTier> for i32
fn tier_to_int(tier: PremiumSlotTier) -> i32 {
    match tier {
        PremiumSlotTier::Lite => 1,
        PremiumSlotTier::Premium => 2,
    }
}

// since the int representation is specific to this postgres implementation, i don't want
// to implement From<PremiumSlotTier> for i32
fn state_to_int(state: PremiumSlotState) -> i32 {
    match state {
        PremiumSlotState::Active => 1,
        PremiumSlotState::Cancelling => 2,
        PremiumSlotState::Cancelled => 3,
        PremiumSlotState::PaymentFailed => 4,
    }
}

struct DbPlugin {
    id: i64,
    created_at: DateTime<Utc>,
    name: String,
    short_description: String,
    long_description: String,
    is_published: bool,
    is_official: bool,
    plugin_kind: i16,
    current_version_number: i32,
    script_published_source: Option<String>,
    script_published_version_updated_at: Option<DateTime<Utc>>,
    script_dev_source: Option<String>,
    script_dev_version_updated_at: Option<DateTime<Utc>>,
    author_id: i64,
    is_public: bool,
}

struct PluginAndImages(DbPlugin, Vec<DbPluginImage>);

impl From<PluginAndImages> for Plugin {
    fn from(PluginAndImages(plugin, images): PluginAndImages) -> Self {
        Self {
            id: plugin.id as u64,
            created_at: plugin.created_at,
            author_id: Id::new(plugin.author_id as u64),
            name: plugin.name,
            short_description: plugin.short_description,
            long_description: plugin.long_description,
            is_public: plugin.is_public,
            is_official: plugin.is_official,
            is_published: plugin.is_published,
            current_version: plugin.current_version_number as u32,
            data: match plugin.plugin_kind {
                0 => PluginData::ScriptPlugin(ScriptPluginData {
                    published_version: plugin.script_published_source,
                    published_version_updated_at: plugin.script_published_version_updated_at,
                    dev_version: plugin.script_dev_source,
                    dev_version_updated_at: plugin.script_dev_version_updated_at,
                }),
                other => {
                    panic!("unknown plugin kind: {other} for plugin id {}", plugin.id)
                }
            },
            images: images.into_iter().map(Into::into).collect(),
        }
    }
}

struct DbUserMeta {
    #[allow(unused)]
    discord_user_id: i64,
    is_moderator: bool,
    is_admin: bool,
    is_verified: bool,
}

impl From<DbUserMeta> for UserMeta {
    fn from(value: DbUserMeta) -> Self {
        Self {
            is_admin: value.is_admin,
            is_moderator: value.is_moderator,
            is_verified: value.is_verified,
        }
    }
}

struct DbPluginImage {
    plugin_id: i64,
    image_id: Uuid,
    created_at: DateTime<Utc>,
    description: String,
    position: i32,
    kind: i32,
    width: i32,
    height: i32,
}

impl From<DbPluginImage> for PluginImage {
    fn from(value: DbPluginImage) -> Self {
        Self {
            plugin_id: value.plugin_id as u64,
            image_id: value.image_id,
            created_at: value.created_at,
            description: value.description,
            position: value.position,
            kind: value.kind.into(),
            width: value.width as u32,
            height: value.height as u32,
        }
    }
}

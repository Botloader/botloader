use async_trait::async_trait;
use chrono::{DateTime, Utc};
use twilight_model::id::{marker::GuildMarker, Id};

use crate::plugins::{
    CreatePluginFields, CreateVersionFields, GuildPluginSubscription, Plugin, PluginStore,
    PluginStoreError, UpdatePluginFields, Version, VersionData, VersionKind, VersionMeta,
    VersionNumber, VersionSelector,
};

use super::Postgres;

#[async_trait]
impl PluginStore for Postgres {
    async fn list_published_plugins(&self) -> Result<Vec<Plugin>, PluginStoreError> {
        Ok(sqlx::query_as!(
            DbPlugin,
            "SELECT id,created_at,author_id,name,short_description,long_description,is_published,\
             is_official FROM plugins WHERE is_published",
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
    }

    async fn get_plugin(&self, id: u64) -> Result<Plugin, PluginStoreError> {
        match sqlx::query_as!(
            DbPlugin,
            "SELECT id,created_at,author_id,name,short_description,long_description,is_published,\
             is_official FROM plugins WHERE id=$1",
            id as i64,
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(s) => Ok(s.into()),
            Err(sqlx::Error::RowNotFound) => Err(PluginStoreError::PluginNotFound),
            Err(e) => Err(e.into()),
        }
    }
    async fn search_plugins(&self, query: String) -> Result<Vec<Plugin>, PluginStoreError> {
        Ok(sqlx::query_as!(
            DbPlugin,
            "SELECT id,created_at,author_id,name,short_description,long_description,is_published,\
             is_official FROM plugins WHERE name ILIKE $1",
            format!("%{query}%"),
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
    }
    async fn create_plugin(&self, plugin: CreatePluginFields) -> Result<Plugin, PluginStoreError> {
        Ok(sqlx::query_as!(
            DbPlugin,
            "INSERT INTO plugins \
             (created_at,author_id,name,short_description,long_description,is_published,\
             is_official)VALUES (now(), $1, $2, $3, $4, $5, $6) 
             RETURNING \
             id,created_at,author_id,name,short_description,long_description,is_published,\
             is_official",
            plugin.author_id.get() as i64,
            plugin.name,
            plugin.short_description,
            plugin.long_description,
            plugin.is_published,
            plugin.is_official,
        )
        .fetch_one(&self.pool)
        .await?
        .into())
    }

    async fn update_plugin(
        &self,
        id: u64,
        plugin: UpdatePluginFields,
    ) -> Result<Plugin, PluginStoreError> {
        match sqlx::query_as!(
            DbPlugin,
            "UPDATE plugins SET
             author_id = COALESCE($2, plugins.author_id),
             name = COALESCE($3, plugins.name),
             short_description = COALESCE($4, plugins.short_description),
             long_description = COALESCE($5, plugins.long_description),
             is_published = COALESCE($6, plugins.is_published),
             is_official = COALESCE($7, plugins.is_official) 
             WHERE id=$1
             RETURNING \
             id,created_at,author_id,name,short_description,long_description,is_published,\
             is_official",
            id as i64,
            plugin.author_id.map(|v| v.get() as i64),
            plugin.name,
            plugin.short_description,
            plugin.long_description,
            plugin.is_published,
            plugin.is_official,
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(s) => Ok(s.into()),
            Err(sqlx::Error::RowNotFound) => Err(PluginStoreError::PluginNotFound),
            Err(e) => Err(e.into()),
        }
    }

    async fn delete_plugin(&self, id: u64) -> Result<Plugin, PluginStoreError> {
        match sqlx::query_as!(
            DbPlugin,
            "DELETE FROM plugins WHERE id=$1 RETURNING \
             id,created_at,author_id,name,short_description,long_description,is_published,\
             is_official",
            id as i64,
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(s) => Ok(s.into()),
            Err(sqlx::Error::RowNotFound) => Err(PluginStoreError::PluginNotFound),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_plugin_version(
        &self,
        plugin_id: u64,
        version: VersionSelector,
    ) -> Result<Version, PluginStoreError> {
        let res = match version {
            VersionSelector::LatestStable => {
                sqlx::query_as!(
                    DbVersion,
                    "SELECT plugin_id,created_at,kind,data,version_major,version_minor FROM \
                     plugin_versions WHERE plugin_id = $1 AND kind=0 ORDER BY version_major DESC, \
                     version_minor DESC limit 1",
                    plugin_id as i64
                )
                .fetch_one(&self.pool)
                .await
            }

            VersionSelector::LatestDevel => {
                sqlx::query_as!(
                    DbVersion,
                    "SELECT plugin_id,created_at,kind,data,version_major,version_minor FROM \
                     plugin_versions WHERE plugin_id = $1 AND (kind=0 OR kind=1 OR kind=2) ORDER \
                     BY version_major DESC, version_minor DESC limit 1",
                    plugin_id as i64,
                )
                .fetch_one(&self.pool)
                .await
            }
            VersionSelector::Pinned(ver) => {
                sqlx::query_as!(
                    DbVersion,
                    "SELECT plugin_id,created_at,kind,data,version_major,version_minor FROM \
                     plugin_versions WHERE plugin_id=$1 AND version_major=$2 AND version_minor=$3",
                    plugin_id as i64,
                    ver.major as i16,
                    ver.minor as i16,
                )
                .fetch_one(&self.pool)
                .await
            }
        };

        match res {
            Ok(s) => Ok(s.into()),
            Err(sqlx::Error::RowNotFound) => Err(PluginStoreError::PluginVersionNotFound),
            Err(e) => Err(e.into()),
        }
    }
    async fn list_plugin_versions(
        &self,
        plugin_id: u64,
    ) -> Result<Vec<VersionMeta>, PluginStoreError> {
        Ok(sqlx::query_as!(
            SlimDbVersion,
            "SELECT plugin_id,created_at,kind,version_major,version_minor FROM plugin_versions \
             WHERE plugin_id=$1 ORDER BY version_major DESC, version_minor DESC",
            plugin_id as i64,
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
    }
    async fn create_plugin_version(
        &self,
        plugin_id: u64,
        fields: CreateVersionFields,
    ) -> Result<Version, PluginStoreError> {
        let data_enc = serde_json::to_value(VersionData {
            sources: fields.sources,
        })
        .unwrap();

        Ok(sqlx::query_as!(
            DbVersion,
            "INSERT INTO plugin_versions \
             (plugin_id,created_at,kind,data,version_major,version_minor)
             VALUES ($1, now(), $2, $3, $4, $5)
             RETURNING plugin_id,created_at,kind,data,version_major,version_minor",
            plugin_id as i64,
            i16::from(fields.kind),
            data_enc,
            fields.number.major as i16,
            fields.number.minor as i16,
        )
        .fetch_one(&self.pool)
        .await?
        .into())
    }
    async fn delete_plugin_version(
        &self,
        plugin_id: u64,
        major: u16,
        minor: u16,
    ) -> Result<Version, PluginStoreError> {
        match sqlx::query_as!(
            DbVersion,
            "DELETE FROM plugin_versions WHERE plugin_id=$1 AND version_major=$2 AND \
             version_minor=$3 
             RETURNING plugin_id,created_at,kind,data,version_major,version_minor",
            plugin_id as i64,
            major as i16,
            minor as i16,
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(s) => Ok(s.into()),
            Err(sqlx::Error::RowNotFound) => Err(PluginStoreError::PluginVersionNotFound),
            Err(e) => Err(e.into()),
        }
    }

    async fn create_update_guild_plugin_subscription(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: u64,
        version: VersionSelector,
    ) -> Result<GuildPluginSubscription, PluginStoreError> {
        let (use_latest_stable, use_devel, pinned_major, pinned_minor) = match version {
            VersionSelector::LatestStable => (true, false, None, None),
            VersionSelector::LatestDevel => (false, true, None, None),
            VersionSelector::Pinned(ver) => {
                (false, false, Some(ver.major as i16), Some(ver.minor as i16))
            }
        };

        Ok(sqlx::query_as!(
            DbGuildPluginSubscription,
            "INSERT INTO guild_plugin_subscriptions \
             (guild_id,plugin_id,pinned_version_major,pinned_version_minor,use_latest_stable,\
             use_devel)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (guild_id, plugin_id) DO UPDATE SET
            pinned_version_major = $3,
            pinned_version_minor = $4,
            use_latest_stable = $5,
            use_devel = $6
            RETURNING \
             guild_id,plugin_id,pinned_version_major,pinned_version_minor,use_latest_stable,\
             use_devel",
            guild_id.get() as i64,
            plugin_id as i64,
            pinned_major,
            pinned_minor,
            use_latest_stable,
            use_devel,
        )
        .fetch_one(&self.pool)
        .await?
        .into())
    }
    async fn list_guild_plugin_subscriptions(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Vec<GuildPluginSubscription>, PluginStoreError> {
        Ok(sqlx::query_as!(
            DbGuildPluginSubscription,
            "SELECT guild_id,plugin_id,pinned_version_major,pinned_version_minor,\
             use_latest_stable,use_devel
                FROM guild_plugin_subscriptions WHERE guild_id=$1",
            guild_id.get() as i64,
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
    }
    async fn delete_guild_plugin_subscription(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: u64,
    ) -> Result<GuildPluginSubscription, PluginStoreError> {
        match sqlx::query_as!(
            DbGuildPluginSubscription,
            "DELETE FROM guild_plugin_subscriptions WHERE guild_id=$1 AND plugin_id=$2 
             RETURNING \
             guild_id,plugin_id,pinned_version_major,pinned_version_minor,use_latest_stable,\
             use_devel",
            guild_id.get() as i64,
            plugin_id as i64,
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(s) => Ok(s.into()),
            Err(sqlx::Error::RowNotFound) => Err(PluginStoreError::GuildPluginSubscriptionNotFound),
            Err(e) => Err(e.into()),
        }
    }
}

struct DbPlugin {
    id: i64,
    created_at: DateTime<Utc>,
    author_id: i64,
    name: String,
    short_description: String,
    long_description: String,
    is_published: bool,
    is_official: bool,
}

impl From<DbPlugin> for Plugin {
    fn from(v: DbPlugin) -> Self {
        Self {
            id: v.id as u64,
            created_at: v.created_at,
            author_id: Id::new(v.author_id as u64),
            name: v.name,
            short_description: v.short_description,
            long_description: v.long_description,
            is_published: v.is_published,
            is_official: v.is_official,
        }
    }
}

struct DbVersion {
    plugin_id: i64,
    created_at: DateTime<Utc>,
    kind: i16,
    data: serde_json::Value,
    version_major: i16,
    version_minor: i16,
}

impl From<DbVersion> for Version {
    fn from(v: DbVersion) -> Self {
        // TODO: rename data to something more descriptive
        // TODO: should we even have data be like this?
        let data = serde_json::from_value(v.data).unwrap();

        Self {
            meta: VersionMeta {
                plugin_id: v.plugin_id as u64,
                created_at: v.created_at,
                kind: v.kind.into(),
                number: VersionNumber {
                    major: v.version_major as u16,
                    minor: v.version_minor as u16,
                },
            },
            data,
        }
    }
}

// DbVersion without source code and other heavy data
struct SlimDbVersion {
    plugin_id: i64,
    created_at: DateTime<Utc>,
    kind: i16,
    version_major: i16,
    version_minor: i16,
}

impl From<SlimDbVersion> for VersionMeta {
    fn from(v: SlimDbVersion) -> Self {
        Self {
            plugin_id: v.plugin_id as u64,
            created_at: v.created_at,
            kind: v.kind.into(),
            number: VersionNumber {
                major: v.version_major as u16,
                minor: v.version_minor as u16,
            },
        }
    }
}

impl From<i16> for VersionKind {
    fn from(v: i16) -> Self {
        match v {
            0 => VersionKind::Stable,
            1 => VersionKind::PreRelease,
            2 => VersionKind::Development,
            other => panic!("unknown version kind {other}"),
        }
    }
}

impl From<VersionKind> for i16 {
    fn from(v: VersionKind) -> Self {
        match v {
            VersionKind::Stable => 0,
            VersionKind::PreRelease => 1,
            VersionKind::Development => 2,
        }
    }
}

struct DbGuildPluginSubscription {
    guild_id: i64,
    plugin_id: i64,
    pinned_version_major: Option<i16>,
    pinned_version_minor: Option<i16>,
    use_latest_stable: bool,
    use_devel: bool,
}

impl From<DbGuildPluginSubscription> for GuildPluginSubscription {
    fn from(v: DbGuildPluginSubscription) -> Self {
        let version = if v.use_devel {
            VersionSelector::LatestDevel
        } else if v.use_latest_stable {
            VersionSelector::LatestStable
        } else if let (Some(major), Some(minor)) = (v.pinned_version_major, v.pinned_version_minor)
        {
            VersionSelector::Pinned(VersionNumber {
                major: major as u16,
                minor: minor as u16,
            })
        } else {
            panic!(
                "no valid version in guild subscription for guild {} and plugin {}!",
                v.guild_id, v.plugin_id
            );
        };

        Self {
            guild_id: Id::new(v.guild_id as u64),
            plugin_id: v.plugin_id as u64,
            version,
        }
    }
}

impl From<sqlx::Error> for PluginStoreError {
    fn from(err: sqlx::Error) -> Self {
        Self::Other(Box::new(err))
    }
}

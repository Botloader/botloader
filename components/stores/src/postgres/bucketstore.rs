use std::time::Duration;

use crate::bucketstore::{Entry, StoreError, StoreResult};

use super::Postgres;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use runtime_models::internal::storage::{
    OpStorageBucketListOrder, OpStorageBucketSetCondition, OpStorageBucketValue,
};
use tracing::error;
use twilight_model::id::{marker::GuildMarker, Id};

#[async_trait]
impl crate::bucketstore::BucketStore for Postgres {
    async fn get(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        key: String,
    ) -> StoreResult<Option<Entry>> {
        let res = sqlx::query_as!(
            DbEntry,
            "SELECT guild_id, plugin_id, bucket, key, created_at, updated_at, expires_at, \
             value_json, value_float FROM bucket_store WHERE guild_id = $1 AND plugin_id = $2 AND \
             bucket = $3 AND key = $4 AND (expires_at IS NULL OR expires_at > now());",
            guild_id.get() as i64,
            plugin_id.unwrap_or(0) as i64,
            bucket,
            key,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(res.map(Into::into))
    }

    async fn set(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        key: String,
        value: OpStorageBucketValue,
        ttl: Option<Duration>,
    ) -> StoreResult<Entry> {
        let expires_at = ttl.and_then(|ttl| {
            chrono::Duration::from_std(ttl)
                .map(|dur| Utc::now() + dur)
                .ok()
        });

        let (val_num, val_json) = match value {
            OpStorageBucketValue::Json(json) => (None, Some(json)),
            OpStorageBucketValue::Double(n) => (Some(n), None),
        };

        let res = sqlx::query_as!(
            DbEntry,
            "INSERT INTO bucket_store
                     (guild_id, plugin_id, bucket, key, created_at, updated_at, expires_at, \
             value_json, value_float)
                     VALUES
                     ($1,         $2,        $3,   $4,     now(),    now(),      $5,            \
             $6,  $7)
                     ON CONFLICT (guild_id, plugin_id, bucket, key) DO UPDATE SET
                     created_at = CASE
                        WHEN bucket_store.expires_at IS NOT NULL AND bucket_store.expires_at < \
             now()
                        THEN now()
                        ELSE bucket_store.created_at
                        END,
                     updated_at = now(),
                     expires_at = excluded.expires_at,
                     value_json = excluded.value_json,
                     value_float = excluded.value_float
                     RETURNING guild_id, plugin_id, bucket, key, created_at, updated_at, \
             expires_at, value_json, value_float;",
            guild_id.get() as i64,
            plugin_id.unwrap_or(0) as i64,
            bucket,
            key,
            expires_at,
            val_json,
            val_num,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.into())
    }
    async fn set_if(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        key: String,
        value: OpStorageBucketValue,
        ttl: Option<Duration>,
        cond: OpStorageBucketSetCondition,
    ) -> StoreResult<Option<Entry>> {
        let expires_at = ttl.and_then(|ttl| {
            chrono::Duration::from_std(ttl)
                .map(|dur| Utc::now() + dur)
                .ok()
        });

        let (val_num, val_json) = match value {
            OpStorageBucketValue::Json(json) => (None, Some(json)),
            OpStorageBucketValue::Double(n) => (Some(n), None),
        };

        let res = match cond {
            OpStorageBucketSetCondition::IfExists => {
                sqlx::query_as!(
                    DbEntry,
                    "UPDATE bucket_store SET
                     updated_at = now(),
                     expires_at = $5,
                     value_json = $6,
                     value_float = $7
                     WHERE guild_id = $1 AND plugin_id = $2 AND bucket = $3 AND key = $4 AND
                     (expires_at IS NULL OR expires_at > now())
                     RETURNING guild_id, plugin_id, bucket, key, created_at, updated_at, \
                     expires_at, value_json, value_float;",
                    guild_id.get() as i64,
                    plugin_id.unwrap_or(0) as i64,
                    bucket,
                    key,
                    expires_at,
                    val_json,
                    val_num,
                )
                .fetch_optional(&self.pool)
                .await
            }
            OpStorageBucketSetCondition::IfNotExists => {
                sqlx::query_as!(
                    DbEntry,
                    "INSERT INTO bucket_store
                    (guild_id, plugin_id, bucket, key, created_at, updated_at, expires_at, \
                     value_json, value_float)
                    VALUES
                    ($1, $2, $3, $4, now(), now(), $5, $6, $7)
                    ON CONFLICT (guild_id, plugin_id, bucket, key) DO UPDATE SET
                    created_at = now(),
                    updated_at = now(),
                    expires_at = excluded.expires_at,
                    value_json = excluded.value_json,
                    value_float = excluded.value_float WHERE
                    (bucket_store.expires_at IS NOT NULL AND bucket_store.expires_at < now())
                    RETURNING guild_id, plugin_id, bucket, key, created_at, updated_at, \
                     expires_at, value_json, value_float;",
                    guild_id.get() as i64,
                    plugin_id.unwrap_or(0) as i64,
                    bucket,
                    key,
                    expires_at,
                    val_json,
                    val_num,
                )
                .fetch_optional(&self.pool)
                .await
            }
        }?;

        Ok(res.map(Into::into))
    }

    async fn del(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        key: String,
    ) -> StoreResult<Option<Entry>> {
        let res = sqlx::query_as!(
            DbEntry,
            "DELETE FROM bucket_store WHERE guild_id = $1 AND plugin_id = $2 AND bucket = $3 AND \
             key = $4 AND (expires_at IS NULL OR expires_at > now()) RETURNING guild_id, \
             plugin_id, bucket, key, created_at, updated_at, expires_at, value_json, value_float;",
            guild_id.get() as i64,
            plugin_id.unwrap_or(0) as i64,
            bucket,
            key,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(res.map(Into::into))
    }

    async fn del_many(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        key_pattern: String,
    ) -> StoreResult<u64> {
        let res = sqlx::query!(
            "DELETE FROM bucket_store WHERE guild_id = $1 AND plugin_id = $2 AND bucket = $3 AND \
             key ILIKE $4 AND (expires_at IS NULL OR expires_at > now());",
            guild_id.get() as i64,
            plugin_id.unwrap_or(0) as i64,
            bucket,
            key_pattern,
        )
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected())
    }

    async fn get_many(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        key_pattern: String,
        after: String,
        limit: u32,
    ) -> StoreResult<Vec<Entry>> {
        let res = sqlx::query_as!(
            DbEntry,
            "SELECT guild_id, plugin_id, bucket, key, created_at, updated_at, expires_at, \
             value_json, value_float FROM bucket_store WHERE guild_id = $1 AND plugin_id = $2 AND \
             bucket = $3 AND key ILIKE $4 AND key > $5 AND (expires_at IS NULL OR expires_at > \
             now()) ORDER BY (guild_id, bucket, key) LIMIT $6;",
            guild_id.get() as i64,
            plugin_id.unwrap_or(0) as i64,
            bucket,
            key_pattern,
            after,
            limit as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(res.into_iter().map(Into::into).collect())
    }

    async fn count(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        key_pattern: String,
    ) -> StoreResult<u64> {
        let res = sqlx::query!(
            "SELECT count(*) FROM bucket_store WHERE guild_id = $1 AND plugin_id = $2 AND bucket \
             = $3 AND key ILIKE $4 AND (expires_at IS NULL OR expires_at > now());",
            guild_id.get() as i64,
            plugin_id.unwrap_or(0) as i64,
            bucket,
            key_pattern,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.count.unwrap_or_default() as u64)
    }

    async fn guild_storage_usage_bytes(&self, guild_id: Id<GuildMarker>) -> StoreResult<u64> {
        let res = sqlx::query!(
            "SELECT sum(pg_column_size(t)) FROM bucket_store t WHERE guild_id=$1 AND (expires_at \
             IS NULL OR expires_at > now())",
            guild_id.get() as i64,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.sum.unwrap_or_default() as u64)
    }

    // the below should only be used for float values
    async fn incr(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        key: String,
        incr_by: f64,
    ) -> StoreResult<Entry> {
        let res = sqlx::query_as!(
            DbEntry,
            "INSERT INTO bucket_store
         (guild_id, plugin_id, bucket, key, created_at, updated_at, expires_at, value_json, \
             value_float)
         VALUES
         ($1, $2, $3, $4, now(), now(), null, null, $5)
         ON CONFLICT (guild_id, plugin_id, bucket, key) DO UPDATE SET
         created_at = CASE
            WHEN bucket_store.expires_at IS NOT NULL AND bucket_store.expires_at < now()
            THEN now()
            ELSE bucket_store.created_at
            END,
         updated_at = now(),
         expires_at = excluded.expires_at,
         value_json = excluded.value_json,
         value_float = CASE
            WHEN bucket_store.expires_at IS NOT NULL AND bucket_store.expires_at < now()
            THEN excluded.value_float
            ELSE excluded.value_float + bucket_store.value_float
            END
         RETURNING guild_id, plugin_id, bucket, key, created_at, updated_at, expires_at, \
             value_json, value_float;",
            guild_id.get() as i64,
            plugin_id.unwrap_or(0) as i64,
            bucket,
            key,
            incr_by,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.into())
    }
    async fn sorted_entries(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        order: OpStorageBucketListOrder,
        offset: u32,
        limit: u32,
    ) -> StoreResult<Vec<Entry>> {
        let res = match order {
            OpStorageBucketListOrder::Ascending => {
                sqlx::query_as!(
                    DbEntry,
                    "SELECT guild_id, plugin_id, bucket, key, created_at, updated_at, expires_at, \
                     value_json, value_float FROM bucket_store WHERE guild_id = $1 AND plugin_id \
                     = $2 AND bucket = $3 AND (expires_at IS NULL OR expires_at > now()) ORDER BY \
                     value_float ASC, updated_at ASC LIMIT $4 OFFSET $5;",
                    guild_id.get() as i64,
                    plugin_id.unwrap_or(0) as i64,
                    bucket,
                    limit as i64,
                    offset as i64,
                )
                .fetch_all(&self.pool)
                .await
            }
            OpStorageBucketListOrder::Descending => {
                sqlx::query_as!(
                    DbEntry,
                    "SELECT guild_id, plugin_id, bucket, key, created_at, updated_at, expires_at, \
                     value_json, value_float FROM bucket_store WHERE guild_id = $1 AND plugin_id \
                     = $2 AND bucket = $3 AND (expires_at IS NULL OR expires_at > now()) ORDER BY \
                     value_float DESC, updated_at DESC LIMIT $4 OFFSET $5;",
                    guild_id.get() as i64,
                    plugin_id.unwrap_or(0) as i64,
                    bucket,
                    limit as i64,
                    offset as i64,
                )
                .fetch_all(&self.pool)
                .await
            }
        }?;

        Ok(res.into_iter().map(Into::into).collect())
    }

    async fn delete_guild_bucket_store_data(&self, id: Id<GuildMarker>) -> StoreResult<()> {
        sqlx::query!(
            "DELETE FROM bucket_store WHERE guild_id = $1",
            id.get() as i64
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[allow(dead_code)]
pub struct DbEntry {
    guild_id: i64,
    plugin_id: i64,
    bucket: String,
    key: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    value_json: Option<serde_json::Value>,
    value_float: Option<f64>,
}

impl From<DbEntry> for Entry {
    fn from(v: DbEntry) -> Self {
        Self {
            bucket: v.bucket,
            plugin_id: (v.plugin_id > 0).then_some(v.plugin_id as u64),
            key: v.key,
            expires_at: v.expires_at,
            value: if let Some(fv) = v.value_float {
                OpStorageBucketValue::Double(fv)
            } else if let Some(sv) = v.value_json {
                OpStorageBucketValue::Json(sv)
            } else {
                error!("got neither float nor json value from db");
                OpStorageBucketValue::Json(serde_json::Value::Null)
            },
        }
    }
}

impl From<sqlx::Error> for StoreError {
    fn from(err: sqlx::Error) -> Self {
        Self::Other(Box::new(err))
    }
}

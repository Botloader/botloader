use std::time::Duration;

use crate::bucketstore::{Entry, SetCondition, SortedOrder, StoreError, StoreResult, StoreValue};

use super::Postgres;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
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
            "SELECT guild_id, bucket, key, created_at, updated_at, expires_at, value_json, \
             value_float FROM bucket_store WHERE guild_id = $1 AND bucket = $2 AND key = $3 AND \
             (expires_at IS NULL OR expires_at > now());",
            guild_id.get() as i64,
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
        value: StoreValue,
        ttl: Option<Duration>,
    ) -> StoreResult<Entry> {
        let expires_at = ttl.and_then(|ttl| {
            chrono::Duration::from_std(ttl)
                .map(|dur| Utc::now() + dur)
                .ok()
        });

        let (val_num, val_json) = match value {
            StoreValue::Json(json) => (None, Some(json)),
            StoreValue::Float(n) => (Some(n), None),
        };

        let res = sqlx::query_as!(
            DbEntry,
            "INSERT INTO bucket_store 
                     (guild_id, bucket, key, created_at, updated_at, expires_at, value_json, \
             value_float)
                     VALUES 
                     ($1,         $2,    $3,   now(),      now(),      $4,         $5,         $6) 
                     ON CONFLICT (guild_id, bucket, key) DO UPDATE SET
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
                     RETURNING guild_id, bucket, key, created_at, updated_at, expires_at, \
             value_json, value_float;",
            guild_id.get() as i64,
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
        value: StoreValue,
        ttl: Option<Duration>,
        cond: SetCondition,
    ) -> StoreResult<Option<Entry>> {
        let expires_at = ttl.and_then(|ttl| {
            chrono::Duration::from_std(ttl)
                .map(|dur| Utc::now() + dur)
                .ok()
        });

        let (val_num, val_json) = match value {
            StoreValue::Json(json) => (None, Some(json)),
            StoreValue::Float(n) => (Some(n), None),
        };

        let res = match cond {
            SetCondition::IfExists => {
                sqlx::query_as!(
                    DbEntry,
                    "UPDATE bucket_store SET
                     updated_at = now(),
                     expires_at = $4,
                     value_json = $5,
                     value_float = $6
                     WHERE guild_id = $1 AND bucket = $2 AND key = $3 AND
                     (expires_at IS NULL OR expires_at > now())
                     RETURNING guild_id, bucket, key, created_at, updated_at, expires_at, \
                     value_json, value_float;",
                    guild_id.get() as i64,
                    bucket,
                    key,
                    expires_at,
                    val_json,
                    val_num,
                )
                .fetch_optional(&self.pool)
                .await
            }
            SetCondition::IfNotExists => {
                sqlx::query_as!(
                    DbEntry,
                    "INSERT INTO bucket_store 
                    (guild_id, bucket, key, created_at, updated_at, expires_at, value_json, \
                     value_float)
                    VALUES 
                    ($1,         $2,    $3,   now(),      now(),      $4,         $5,         $6) 
                    ON CONFLICT (guild_id, bucket, key) DO UPDATE SET
                    created_at = now(),
                    updated_at = now(),
                    expires_at = excluded.expires_at,
                    value_json = excluded.value_json,
                    value_float = excluded.value_float WHERE 
                    (bucket_store.expires_at IS NOT NULL AND bucket_store.expires_at < now())
                    RETURNING guild_id, bucket, key, created_at, updated_at, expires_at, \
                     value_json, value_float;",
                    guild_id.get() as i64,
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
            "DELETE FROM bucket_store WHERE guild_id = $1 AND bucket = $2 AND key = $3 AND \
             (expires_at IS NULL OR expires_at > now()) RETURNING guild_id, bucket, key, \
             created_at, updated_at, expires_at, value_json, value_float;",
            guild_id.get() as i64,
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
            "DELETE FROM bucket_store WHERE guild_id = $1 AND bucket = $2 AND key ILIKE $3 AND \
             (expires_at IS NULL OR expires_at > now());",
            guild_id.get() as i64,
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
            "SELECT guild_id, bucket, key, created_at, updated_at, expires_at, value_json, \
             value_float FROM bucket_store WHERE guild_id = $1 AND bucket = $2 AND key ILIKE $3 \
             AND key > $4 AND (expires_at IS NULL OR expires_at > now()) ORDER BY (guild_id, \
             bucket, key) LIMIT $5;",
            guild_id.get() as i64,
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
            "SELECT count(*) FROM bucket_store WHERE guild_id = $1 AND bucket = $2 AND key ILIKE \
             $3 AND (expires_at IS NULL OR expires_at > now());",
            guild_id.get() as i64,
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
         (guild_id, bucket, key, created_at, updated_at, expires_at, value_json, value_float)
         VALUES 
         ($1,         $2,    $3,   now(),      now(),      null,         null,         $4) 
         ON CONFLICT (guild_id, bucket, key) DO UPDATE SET
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
         RETURNING guild_id, bucket, key, created_at, updated_at, expires_at, value_json, \
             value_float;",
            guild_id.get() as i64,
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
        order: SortedOrder,
        offset: u32,
        limit: u32,
    ) -> StoreResult<Vec<Entry>> {
        let res = match order {
            SortedOrder::Ascending => {
                sqlx::query_as!(
                    DbEntry,
                    "SELECT guild_id, bucket, key, created_at, updated_at, expires_at, \
                     value_json, value_float FROM bucket_store WHERE guild_id = $1 AND bucket = \
                     $2 AND (expires_at IS NULL OR expires_at > now()) ORDER BY value_float ASC, \
                     updated_at ASC LIMIT $3 OFFSET $4;",
                    guild_id.get() as i64,
                    bucket,
                    limit as i64,
                    offset as i64,
                )
                .fetch_all(&self.pool)
                .await
            }
            SortedOrder::Descending => {
                sqlx::query_as!(
                    DbEntry,
                    "SELECT guild_id, bucket, key, created_at, updated_at, expires_at, \
                     value_json, value_float FROM bucket_store WHERE guild_id = $1 AND bucket = \
                     $2 AND (expires_at IS NULL OR expires_at > now()) ORDER BY value_float DESC, \
                     updated_at DESC LIMIT $3 OFFSET $4;",
                    guild_id.get() as i64,
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
            key: v.key,
            expires_at: v.expires_at,
            value: if let Some(fv) = v.value_float {
                StoreValue::Float(fv)
            } else if let Some(sv) = v.value_json {
                StoreValue::Json(sv)
            } else {
                error!("got neither float nor json value from db");
                StoreValue::Json(serde_json::Value::Null)
            },
        }
    }
}

impl From<sqlx::Error> for StoreError {
    fn from(err: sqlx::Error) -> Self {
        Self::Other(Box::new(err))
    }
}

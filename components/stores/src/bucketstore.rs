use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;
use twilight_model::id::{marker::GuildMarker, Id};

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("guild storage capacity reached")]
    GuildStorageLimitReached,

    #[error("inner error occured: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub type StoreResult<T> = Result<T, StoreError>;

#[async_trait]
pub trait BucketStore: Send + Sync {
    async fn get(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        key: String,
    ) -> StoreResult<Option<Entry>>;

    async fn set(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        key: String,
        value: StoreValue,
        ttl: Option<Duration>,
    ) -> StoreResult<Entry>;

    async fn set_if(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        key: String,
        value: StoreValue,
        ttl: Option<Duration>,
        cond: SetCondition,
    ) -> StoreResult<Option<Entry>>;

    async fn del(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        key: String,
    ) -> StoreResult<Option<Entry>>;

    async fn del_many(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        key_pattern: String,
    ) -> StoreResult<u64>;

    async fn get_many(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        key_pattern: String,
        after: String,
        limit: u32,
    ) -> StoreResult<Vec<Entry>>;

    async fn count(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        key_pattern: String,
    ) -> StoreResult<u64>;

    async fn guild_storage_usage_bytes(&self, guild_id: Id<GuildMarker>) -> StoreResult<u64>;

    // the below should only be used for float values
    async fn incr(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        key: String,
        incr_by: f64,
    ) -> StoreResult<Entry>;

    async fn sorted_entries(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        bucket: String,
        order: SortedOrder,
        offset: u32,
        limit: u32,
    ) -> StoreResult<Vec<Entry>>;

    async fn delete_guild_bucket_store_data(&self, id: Id<GuildMarker>) -> StoreResult<()>;
}

pub enum SetCondition {
    IfNotExists,
    IfExists,
}

pub enum SortedOrder {
    Ascending,
    Descending,
}

#[derive(Debug)]
pub struct Entry {
    pub bucket: String,
    pub key: String,
    pub value: StoreValue,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug)]
pub enum StoreValue {
    Json(serde_json::Value),
    Float(f64),
}

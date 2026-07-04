pub mod bucketstore;
pub mod config;
pub mod inmemory;
pub mod timers;
pub mod web;

use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    pub fn new_with_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn new_with_url(url: &str) -> Result<Self, anyhow::Error> {
        let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;

        Ok(Self { pool })
    }

    /// Creates a Db without connecting, connections are only established when used.
    ///
    /// Useful when you need a Db instance but never issue queries (e.g. benchmarks).
    pub fn new_with_url_lazy(url: &str) -> Result<Self, anyhow::Error> {
        let pool = PgPoolOptions::new().max_connections(5).connect_lazy(url)?;

        Ok(Self { pool })
    }
}

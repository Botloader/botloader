pub mod bucketstore;
pub mod config;
pub mod inmemory;
pub mod timers;

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
}

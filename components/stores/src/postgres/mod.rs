use sqlx::{postgres::PgPoolOptions, PgPool};

pub mod bucketstore;
pub mod config;
pub mod plugins;
pub mod timers;
pub mod web;

#[derive(Clone)]
pub struct Postgres {
    pool: PgPool,
}

impl Postgres {
    pub fn new_with_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn new_with_url(url: &str) -> Result<Self, anyhow::Error> {
        let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;

        Ok(Self { pool })
    }
}

use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

pub struct Db {
    conn: DatabaseConnection,
}

impl Db {
    pub async fn connect(url: String) -> Result<Self, DbErr> {
        let mut opt = ConnectOptions::new(url);
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            // .sqlx_logging(true)
            .set_schema_search_path("public"); // Setting default PostgreSQL schema1

        let db = Database::connect(opt).await?;
        Ok(Self { conn: db })
    }
}

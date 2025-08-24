use anyhow::Context;
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub bind_addr: String,
    pub database_url: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8000".into());
        let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL missing")?;
        Ok(Self { bind_addr, database_url })
    }

    pub async fn make_db_pool(&self) -> anyhow::Result<PgPool> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect(&self.database_url)
            .await?;
        Ok(pool)
    }
}

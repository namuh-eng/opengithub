use std::time::Duration;

use anyhow::{Context, Result};
use sqlx::{postgres::PgPoolOptions, PgPool};
use url::Url;

pub type DbPool = PgPool;

pub async fn pool_from_env() -> Result<Option<DbPool>> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_default();
    if database_url.trim().is_empty() {
        return Ok(None);
    }

    connect(&database_url).await.map(Some)
}

pub async fn connect(database_url: &str) -> Result<DbPool> {
    PgPoolOptions::new()
        .max_connections(max_connections())
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url_with_ssl_mode(database_url)?)
        .await
        .context("failed to connect to Postgres")
}

pub fn test_pool_options() -> PgPoolOptions {
    PgPoolOptions::new()
        .max_connections(8)
        .acquire_timeout(Duration::from_secs(30))
}

fn max_connections() -> u32 {
    std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(5)
}

fn database_url_with_ssl_mode(database_url: &str) -> Result<String> {
    let db_ssl = std::env::var("DB_SSL").unwrap_or_default();
    if db_ssl.trim().is_empty() || database_url.contains("sslmode=") {
        return Ok(database_url.to_owned());
    }

    let mut url = Url::parse(database_url).context("DATABASE_URL must be a valid URL")?;
    url.query_pairs_mut().append_pair(
        "sslmode",
        if db_ssl.eq_ignore_ascii_case("true") {
            "require"
        } else {
            "disable"
        },
    );
    Ok(url.to_string())
}

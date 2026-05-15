use std::path::Path;

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use crate::error::AppResult;

pub type DbPool = SqlitePool;

#[derive(Clone)]
pub struct DbState {
    pub pool: DbPool,
}

pub async fn connect_database(database_path: &Path) -> AppResult<DbPool> {
    if let Some(parent) = database_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let database_url = format!("sqlite://{}?mode=rwc", database_path.to_string_lossy());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    configure_pool(&pool).await?;
    run_migrations(&pool).await?;
    cleanup_expired_sessions(&pool).await?;

    Ok(pool)
}

pub async fn create_test_pool() -> AppResult<DbPool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;

    configure_pool(&pool).await?;
    run_migrations(&pool).await?;
    Ok(pool)
}

async fn configure_pool(pool: &DbPool) -> AppResult<()> {
    sqlx::query("PRAGMA foreign_keys=ON").execute(pool).await?;
    sqlx::query("PRAGMA journal_mode=WAL").execute(pool).await?;
    sqlx::query("PRAGMA synchronous=NORMAL")
        .execute(pool)
        .await?;
    sqlx::query("PRAGMA busy_timeout=5000")
        .execute(pool)
        .await?;
    Ok(())
}

async fn run_migrations(pool: &DbPool) -> AppResult<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

pub async fn cleanup_expired_sessions(pool: &DbPool) -> AppResult<()> {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    sqlx::query!("DELETE FROM session WHERE expires_at <= ?", now)
        .execute(pool)
        .await?;
    Ok(())
}

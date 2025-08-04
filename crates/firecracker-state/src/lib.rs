use anyhow::{anyhow, Context, Error};
use sqlx::{sqlite::SqliteConnectOptions, Executor, Pool, Sqlite, SqlitePool};
use std::fs;

pub mod entity;
pub mod repo;

pub async fn create_connection_pool() -> Result<Pool<Sqlite>, Error> {
    let config_dir = get_config_dir()?;
    let db_path = format!("{}/firecracker_state.db", config_dir);
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;

    pool.execute(include_str!("../migrations/20250804092946_init.sql"))
        .await?;

    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;

    Ok(pool)
}

fn get_config_dir() -> Result<String, Error> {
    let app_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("Failed to get home directory"))?
        .join(".fireup");
    fs::create_dir_all(&app_dir)
        .with_context(|| format!("Failed to create app directory: {}", app_dir.display()))?;

    Ok(app_dir.display().to_string())
}

use anyhow::{Context, Error};
use sqlx::{Pool, Sqlite};

use crate::entity::drive::Drive;

pub async fn all(pool: &Pool<Sqlite>) -> Result<Vec<Drive>, Error> {
    let result: Vec<Drive> = sqlx::query_as("SELECT * FROM drives")
        .fetch_all(pool)
        .await
        .with_context(|| "Failed to fetch drives")?;
    Ok(result)
}

pub async fn find(pool: &Pool<Sqlite>, name: &str) -> Result<Option<Drive>, Error> {
    let result: Option<Drive> = sqlx::query_as("SELECT * FROM drives WHERE name = ? OR id = ?")
        .bind(name)
        .bind(name)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("Failed to find drive with name or id '{}'", name))?;
    Ok(result)
}

pub async fn find_by_vm_id(pool: &Pool<Sqlite>, vm_id: &str) -> Result<Vec<Drive>, Error> {
    let result: Vec<Drive> = sqlx::query_as("SELECT * FROM drives WHERE vm_id = ?")
        .bind(vm_id)
        .fetch_all(pool)
        .await
        .with_context(|| format!("Failed to find drives for VM with id '{}'", vm_id))?;
    Ok(result)
}

pub async fn create(pool: &Pool<Sqlite>, drive: &Drive) -> Result<(), Error> {
    sqlx::query("INSERT INTO drives (id, name, vm_id, path_on_host, is_root_device, is_read_only, size_in_gb) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&drive.id)
        .bind(&drive.name)
        .bind(&drive.vm_id)
        .bind(&drive.path_on_host)
        .bind(drive.is_root_device)
        .bind(drive.is_read_only)
        .bind(drive.size_in_gb)
        .execute(pool)
        .await
        .with_context(|| format!("Failed to create drive with name '{}'", drive.name))?;
    Ok(())
}

pub async fn delete(pool: &Pool<Sqlite>, name: &str) -> Result<(), Error> {
    sqlx::query("DELETE FROM drives WHERE name = ? OR id = ?")
        .bind(name)
        .bind(name)
        .execute(pool)
        .await
        .with_context(|| format!("Failed to delete drive with name or id '{}'", name))?;
    Ok(())
}

pub async fn update_vm_id(
    pool: &Pool<Sqlite>,
    drive_id: &str,
    vm_id: Option<String>,
) -> Result<(), Error> {
    sqlx::query(
        "UPDATE drives SET vm_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? OR name = ?",
    )
    .bind(vm_id)
    .bind(drive_id)
    .bind(drive_id)
    .execute(pool)
    .await
    .with_context(|| format!("Failed to update vm_id for drive with id '{}'", drive_id))?;
    Ok(())
}

pub async fn update_name(pool: &Pool<Sqlite>, drive_id: &str, new_name: &str) -> Result<(), Error> {
    sqlx::query(
        "UPDATE drives SET name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? OR name = ?",
    )
    .bind(new_name)
    .bind(drive_id)
    .bind(drive_id)
    .execute(pool)
    .await
    .with_context(|| format!("Failed to update name for drive with id '{}'", drive_id))?;
    Ok(())
}

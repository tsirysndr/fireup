use std::path::Path;

use anyhow::{Context, Error};
use sqlx::{Pool, Sqlite};

use crate::entity::virtual_machine::VirtualMachine;

pub async fn all(pool: Pool<Sqlite>) -> Result<Vec<VirtualMachine>, Error> {
    let result: Vec<VirtualMachine> = sqlx::query_as("SELECT * FROM virtual_machines")
        .fetch_all(&pool)
        .await
        .with_context(|| "Failed to fetch virtual machines")?;
    Ok(result)
}

pub async fn find(pool: Pool<Sqlite>, name: &str) -> Result<Option<VirtualMachine>, Error> {
    let result: Option<VirtualMachine> =
        sqlx::query_as("SELECT * FROM virtual_machines WHERE name = ? OR id = ?")
            .bind(name)
            .fetch_optional(&pool)
            .await
            .with_context(|| {
                format!("Failed to find virtual machine with name or id '{}'", name)
            })?;
    Ok(result)
}

pub async fn find_by_project_dir(
    pool: Pool<Sqlite>,
    path: &str,
) -> Result<Option<VirtualMachine>, Error> {
    let result: Option<VirtualMachine> =
        sqlx::query_as("SELECT * FROM virtual_machines WHERE project_dir = ?")
            .bind(path)
            .fetch_optional(&pool)
            .await
            .with_context(|| {
                format!("Failed to find virtual machine with project_dir '{}'", path)
            })?;
    Ok(result)
}

pub async fn create(pool: Pool<Sqlite>, vm: VirtualMachine) -> Result<(), Error> {
    let id = xid::new().to_string();
    let project_dir = match Path::exists(Path::new("fire.toml")) {
      true => Some(std::env::current_dir()?.display().to_string()),
      false => None,
    };
    sqlx::query("INSERT INTO virtual_machines (
      name,
      id,
      project_dir,
      bridge,
      tap,
      api_socket,
      mac_address,
      vcpu,
      memory,
      distro,
      pid,
      status
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&vm.name)
        .bind(&id)
        .bind(project_dir)
        .bind(&vm.bridge)
        .bind(&vm.tap)
        .bind(&vm.api_socket)
        .bind(&vm.mac_address)
        .bind(&vm.vcpu)
        .bind(&vm.memory)
        .bind(&vm.distro)
        .bind(&vm.pid)
        .bind("RUNNING")
        .execute(&pool)
        .await
        .with_context(|| "Failed to create virtual machine")?;
    Ok(())
}

pub async fn delete(pool: Pool<Sqlite>, name: &str) -> Result<(), Error> {
    sqlx::query("DELETE FROM virtual_machines WHERE name = ? OR id = ?")
        .bind(name)
        .bind(name)
        .execute(&pool)
        .await
        .with_context(|| {
            format!(
                "Failed to delete virtual machine with name or id '{}'",
                name
            )
        })?;
    Ok(())
}

pub async fn update(pool: Pool<Sqlite>, vm: VirtualMachine, status: &str) -> Result<(), Error> {
    sqlx::query("UPDATE virtual_machines SET project_dir = ?, bridge = ?, tap = ?, api_socket = ?, mac_address = ? WHERE name = ? OR id = ?")
        .bind(&vm.project_dir)
        .bind(&vm.bridge)
        .bind(&vm.tap)
        .bind(&vm.api_socket)
        .bind(&vm.mac_address)
        .bind(&vm.name)
        .bind(&vm.id)
        .bind(status)
        .execute(&pool)
        .await
        .with_context(|| format!("Failed to update virtual machine with name or id '{}'", vm.name))?;
    Ok(())
}

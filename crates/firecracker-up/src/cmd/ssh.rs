use crate::{command::run_command, config::get_config_dir};
use anyhow::Error;
use firecracker_state::repo;
use glob::glob;
use sqlx::{Pool, Sqlite};

pub async fn ssh(pool: Pool<Sqlite>, name: Option<String>) -> Result<(), Error> {
    let guest_ip = match name {
        Some(name) => format!("{}.firecracker.local", name),
        None => {
            let current_dir = std::env::current_dir()
                .map_err(|e| Error::msg(format!("Failed to get current directory: {}", e)))?
                .display()
                .to_string();
            let vm = repo::virtual_machine::find_by_project_dir(pool, &current_dir).await?;
            match vm {
                Some(vm) => format!("{}.firecracker.local", vm.name),
                None => {
                    return Err(Error::msg(
                        "No virtual machine found with the given name or project directory.",
                    ))
                }
            }
        }
    };
    let app_dir = get_config_dir()?;
    let private_key = glob(format!("{}/id_rsa", app_dir).as_str())
        .map_err(|e| Error::msg(format!("Failed to find SSH key: {}", e)))?
        .last()
        .ok_or_else(|| Error::msg("No SSH key file found"))?;
    run_command(
        "ssh",
        &[
            "-i",
            &private_key?.display().to_string(),
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "UserKnownHostsFile=/dev/null",
            &format!("root@{}", guest_ip),
        ],
        true,
    )?;
    Ok(())
}

use anyhow::{anyhow, Context, Error, Result};
use firecracker_state::repo;
use owo_colors::OwoColorize;
use std::fs;

use crate::command::run_ssh_command;

pub async fn exec(name: &str, args: Vec<String>) -> Result<(), Error> {
    let pool = firecracker_state::create_connection_pool().await?;
    let vm = repo::virtual_machine::find(&pool, name).await?;

    if vm.is_none() {
        println!("[-] MicroVM '{}' not found.", name);
        std::process::exit(1);
    }

    if !firecracker_process::vm_is_running(name).await? {
        println!("[-] MicroVM '{}' is not running.", name);
        let start_cmd = format!("fireup start {}", name);
        println!("    Start it with {}", start_cmd.cyan());
        std::process::exit(1);
    }

    let guest_ip = format!("{}.firecracker", name);
    run_ssh_command(&get_private_key_path()?, &guest_ip, args.join(" ").as_str())?;

    Ok(())
}

fn get_private_key_path() -> Result<String, Error> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Failed to get home directory"))?;
    let app_dir = format!("{}/.fireup", home_dir.display());
    let key_name = glob::glob(format!("{}/id_rsa", app_dir).as_str())
        .with_context(|| "Failed to glob ssh key files")?
        .last()
        .ok_or_else(|| anyhow!("No SSH key file found"))?
        .with_context(|| "Failed to get SSH key path")?;
    let key_name = fs::canonicalize(&key_name)
        .with_context(|| {
            format!(
                "Failed to resolve absolute path for SSH key: {}",
                key_name.display()
            )
        })?
        .display()
        .to_string();
    Ok(key_name)
}

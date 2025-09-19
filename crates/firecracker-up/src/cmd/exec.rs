use anyhow::{Error, Result};
use firecracker_state::repo;
use owo_colors::OwoColorize;

use crate::{command::run_ssh_command, ssh::get_private_key_path};

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

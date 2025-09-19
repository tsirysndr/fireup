use anyhow::Error;
use owo_colors::OwoColorize;

use crate::{command::run_command_with_stdout_inherit, ssh::get_private_key_path};

pub async fn cp(from: &str, to: &str) -> Result<(), Error> {
    let pool = firecracker_state::create_connection_pool().await?;

    let vm_name = if from.contains(':') {
        from.split(':').next().unwrap_or("")
    } else if to.contains(':') {
        to.split(':').next().unwrap_or("")
    } else {
        return Err(anyhow::anyhow!(
            "Either source or destination must be in the format <vm_name>:<path>"
        ));
    };

    let vm = firecracker_state::repo::virtual_machine::find(&pool, vm_name).await?;

    if vm.is_none() {
        println!("[-] MicroVM '{}' not found.", vm_name);
        std::process::exit(1);
    }

    if !firecracker_process::vm_is_running(vm_name).await? {
        println!("[-] MicroVM '{}' is not running.", vm_name);
        let start_cmd = format!("fireup start {}", vm_name);
        println!("    Start it with {}", start_cmd.cyan());
        std::process::exit(1);
    }

    let guest_ip = format!("{}.firecracker", vm_name);
    let key_path = get_private_key_path()?;

    let scp_args = if from.contains(':') {
        let remote_path = format!("root@{}:{}", guest_ip, from.splitn(2, ':').nth(1).unwrap());
        vec!["-r", remote_path.as_str(), to]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    } else {
        let remote_path = format!("root@{}:{}", guest_ip, to.splitn(2, ':').nth(1).unwrap());
        vec!["-r", from, remote_path.as_str()]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    };

    run_command_with_stdout_inherit(
        "scp",
        &[
            "-q",
            "-i",
            &key_path,
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "UserKnownHostsFile=/dev/null",
        ]
        .iter()
        .copied()
        .chain(scp_args.iter().map(|s| s.as_str()))
        .collect::<Vec<&str>>()
        .as_slice(),
        false,
    )?;

    Ok(())
}

use std::{process, thread};

use anyhow::Result;
use firecracker_state::repo;
use firecracker_vm::types::VmOptions;
use owo_colors::OwoColorize;

use crate::command::{run_command, run_command_in_background};

pub mod command;

pub async fn start(config: &VmOptions) -> Result<u32> {
    let name = config
        .api_socket
        .trim_start_matches("/tmp/firecracker-")
        .trim_end_matches(".sock")
        .to_string();

    stop(Some(name)).await?;
    println!("[+] Starting Firecracker...");
    let pid = run_command_in_background("firecracker", &["--api-sock", &config.api_socket], true)?;

    let mut attempts = 0;
    while !std::path::Path::new(&config.api_socket).exists() {
        if attempts >= 100 {
            println!("[!] Timed out waiting for Firecracker to start. Please check the logs.");
            process::exit(1);
        }
        attempts += 1;
        thread::sleep(std::time::Duration::from_millis(500));
    }

    Ok(pid)
}

pub async fn stop(name: Option<String>) -> Result<()> {
    if name.is_none() {
        return stop_all().await;
    }

    let name = name.unwrap();

    if !vm_is_running(&name).await? {
        println!("[!] {} is not running.", name.cyan());
        return Ok(());
    }

    let config = VmOptions {
        api_socket: format!("/tmp/firecracker-{}.sock", name),
        ..Default::default()
    };

    let pool = firecracker_state::create_connection_pool().await?;

    let vm = repo::virtual_machine::find(&pool, &name).await?;
    if vm.is_none() {
        println!(
            "[!] No virtual machine found with name or id '{}'.",
            name.cyan()
        );
        process::exit(1);
    }

    let vm = vm.unwrap();
    if let Some(pid) = vm.pid {
        if run_command("kill", &["-s", "KILL", &pid.to_string()], true).is_err() {
            println!("[!] Failed to kill process with PID {}.", pid);
        }
    }

    run_command("rm", &["-rf", &config.api_socket], true)?;
    println!("[+] {} has been stopped.", name.cyan());

    repo::virtual_machine::update_status(&pool, &name, "STOPPED").await?;

    Ok(())
}

pub async fn vm_is_running(name: &str) -> Result<bool> {
    let pool = firecracker_state::create_connection_pool().await?;
    let vm = repo::virtual_machine::find(&pool, name).await?;

    if let Some(vm) = vm {
        if std::path::Path::new(&vm.api_socket).exists() {
            return Ok(true);
        }
        repo::virtual_machine::update_status(&pool, name, "STOPPED").await?;
    }

    Ok(false)
}

pub fn is_running() -> bool {
    match run_command("pgrep", &["-x", "firecracker"], false) {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

pub async fn stop_all() -> Result<()> {
    if !is_running() {
        println!("[!] No Firecracker process is running.");
        return Ok(());
    }

    run_command("pkill", &["-x", "firecracker"], true)?;
    run_command("bash", &["-c", "rm -rf /tmp/firecracker-*.sock"], true)?;
    println!("[+] All Firecracker processes have been stopped.");

    let pool = firecracker_state::create_connection_pool().await?;
    repo::virtual_machine::update_all_status(&pool, "STOPPED").await?;
    Ok(())
}

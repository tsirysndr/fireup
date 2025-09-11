use std::{process, thread};

use anyhow::Error;
use fire_config::read_config;
use firecracker_state::repo;
use firecracker_vm::types::VmOptions;
use owo_colors::OwoColorize;

use crate::command::run_command;

pub async fn up(options: VmOptions) -> Result<(), Error> {
    check_kvm_support()?;

    let mut options = match read_config() {
        Ok(config) => VmOptions::from(config),
        Err(_) => options.clone(),
    };

    let current_dir = std::env::current_dir()?;
    let fire_toml = current_dir.join("fire.toml");
    let mut vm_id = None;
    let pool = firecracker_state::create_connection_pool().await?;
    let vm = repo::virtual_machine::find_by_api_socket(&pool, &options.api_socket).await?;

    if let Some(vm) = vm {
        vm_id = Some(vm.id.clone());
    }

    if fire_toml.exists() {
        let vm =
            repo::virtual_machine::find_by_project_dir(&pool, &current_dir.display().to_string())
                .await?;

        if let Some(vm) = vm {
            options.api_socket = vm.api_socket.clone();
            vm_id = Some(vm.id.clone());
        }
    }

    let vms = repo::virtual_machine::all(&pool).await?;
    if options.tap.is_empty() {
        let vms = vms
            .into_iter()
            .filter(|vm| vm.tap.starts_with("tap"))
            .collect::<Vec<_>>();
        options.tap = format!("tap{}", vms.len());

        while vms.iter().any(|vm| vm.tap == options.tap) {
            let tap_num: u32 = options
                .tap
                .trim_start_matches("tap")
                .parse::<u32>()
                .unwrap_or(0)
                .checked_add(1)
                .unwrap_or(0);
            options.tap = format!("tap{}", tap_num);
        }
    } else {
        if vms
            .iter()
            .any(|vm| vm.tap == options.tap && vm.api_socket != options.api_socket)
        {
            println!(
                "[!] Tap device name {} is already in use. Please choose a different name.",
                options.tap.cyan()
            );
            process::exit(1);
        }
    }

    let pid = firecracker_process::start(&options).await?;

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
        if firecracker_process::is_running() {
            println!("[+] Firecracker is running.");
            break;
        }
    }

    firecracker_prepare::prepare(options.clone().into())?;
    firecracker_vm::setup(&options, pid, vm_id).await?;
    Ok(())
}

pub fn check_kvm_support() -> Result<(), Error> {
    print!("[+] Checking for kvm support... ");

    if !run_command("sh", &["-c", "lsmod | grep kvm"], false)
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        return Err(anyhow::anyhow!(
            "KVM is not available. Please ensure KVM is enabled in your system."
        ));
    }

    println!("{}", "[✓] OK".bright_green());

    Ok(())
}

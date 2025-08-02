use std::thread;

use anyhow::Error;
use fire_config::read_config;
use firecracker_vm::types::VmOptions;
use owo_colors::OwoColorize;

use crate::command::run_command;

pub fn up(options: VmOptions) -> Result<(), Error> {
    check_kvm_support()?;

    let options = match read_config() {
        Ok(config) => VmOptions::from(config),
        Err(_) => options.clone(),
    };

    firecracker_process::start()?;

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
        if firecracker_process::is_running() {
            println!("[+] Firecracker is running.");
            break;
        }
    }

    firecracker_prepare::prepare(options.clone().into())?;
    firecracker_vm::setup(&options)?;
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

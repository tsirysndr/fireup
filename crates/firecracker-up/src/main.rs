use std::thread;

use anyhow::Result;
use owo_colors::OwoColorize;

use crate::command::run_command;

pub mod command;

fn main() -> Result<()> {
    check_kvm_support()?;

    firecracker_process::start()?;

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
        if firecracker_process::is_running() {
            println!("[+] Firecracker is running.");
            break;
        }
    }

    firecracker_prepare::prepare()?;
    firecracker_vm::setup()?;
    Ok(())
}

pub fn check_kvm_support() -> Result<()> {
    print!("[+] Checking for kvm support... ");

    if !run_command("sh", &["-c", "lsmod | grep kvm"])
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

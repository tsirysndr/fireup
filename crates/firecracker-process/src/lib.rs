use anyhow::Result;
use firecracker_vm::types::VmOptions;

use crate::command::{run_command, run_command_in_background};

pub mod command;

pub fn start(config: &VmOptions) -> Result<u32> {
    stop(config)?;
    println!("[+] Starting Firecracker...");
    let pid = run_command_in_background("firecracker", &["--api-sock", &config.api_socket], true)?;
    Ok(pid)
}

pub fn stop(config: &VmOptions) -> Result<()> {
    if !is_running() {
        println!("[!] Firecracker is not running.");
        run_command("rm", &["-rf", &config.api_socket], true)?;
        return Ok(());
    }
    run_command("killall", &["-s", "KILL", "firecracker"], true)?;
    run_command("rm", &["-rf", &config.api_socket], true)?;
    println!("[+] Firecracker has been stopped.");
    Ok(())
}

pub fn is_running() -> bool {
    run_command("pgrep", &["firecracker"], false).is_ok()
}

use anyhow::Result;

use crate::command::{run_command, run_command_in_background};

pub mod command;

pub const FIRECRACKER_SOCKET: &str = "/tmp/firecracker.sock";

pub fn start() -> Result<()> {
    stop()?;
    println!("[+] Starting Firecracker...");
    run_command_in_background("firecracker", &["--api-sock", FIRECRACKER_SOCKET], true)?;
    Ok(())
}

pub fn stop() -> Result<()> {
    if !is_running() {
        println!("[!] Firecracker is not running.");
        run_command("rm", &["-rf", FIRECRACKER_SOCKET], true)?;
        return Ok(());
    }
    run_command("killall", &["-s", "KILL", "firecracker"], true)?;
    run_command("rm", &["-rf", FIRECRACKER_SOCKET], true)?;
    println!("[+] Firecracker stopped.");
    Ok(())
}

pub fn is_running() -> bool {
    run_command("pgrep", &["firecracker"], false).is_ok()
}

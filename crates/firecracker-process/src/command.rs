use anyhow::{anyhow, Context, Error, Result};
use owo_colors::OwoColorize;
use std::{
    process::{Command, Output, Stdio},
    thread,
};

pub fn has_sudo() -> bool {
    Command::new("sudo")
        .arg("-h")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn is_root() -> bool {
    unsafe { libc::getuid() == 0 }
}

pub fn run_command(command: &str, args: &[&str], use_sudo: bool) -> Result<Output> {
    let mut cmd = if use_sudo {
        if !has_sudo() && !is_root() {
            return Err(anyhow!(
                "sudo is required for command '{}', but not available",
                command
            ));
        }
        let mut c = Command::new("sudo");
        c.arg(command);

        match is_root() {
            true => Command::new(command),
            false => c,
        }
    } else {
        Command::new(command)
    };

    let output = cmd
        .args(args)
        .stdin(Stdio::inherit())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("Failed to execute {}", command))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Command {} failed: {}", command, stderr));
    }
    Ok(output)
}

pub fn run_command_in_background(command: &str, args: &[&str], use_sudo: bool) -> Result<u32> {
    let mut cmd = if use_sudo {
        if !has_sudo() && !is_root() {
            return Err(anyhow!(
                "sudo is required for command '{}', but not available",
                command
            ));
        }
        let mut c = Command::new("sudo");
        c.arg(command);

        match is_root() {
            true => Command::new(command),
            false => c,
        }
    } else {
        Command::new(command)
    };

    let command_owned = command.to_string();
    let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    let (tx, rx) = std::sync::mpsc::channel::<u32>();

    thread::spawn(move || {
        let mut child = cmd
            .args(&args_owned)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("Failed to execute {}", command_owned))?;

        let pid = child.id();
        tx.send(pid)
            .with_context(|| format!("Failed to send PID for command {}", command_owned))?;
        println!(
            "[+] Started command {} with PID {}",
            command_owned.bright_cyan(),
            pid.bright_cyan()
        );

        child
            .wait()
            .with_context(|| format!("Failed to wait for command {}", command_owned))?;
        Ok::<(), Error>(())
    });

    let pid = rx
        .recv()
        .with_context(|| format!("Failed to receive PID for command {}", command))?;

    Ok(pid)
}

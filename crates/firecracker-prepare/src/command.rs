use anyhow::{anyhow, Context, Result};
use std::process::{Command, Output, Stdio};

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

pub fn run_command_with_stdout_inherit(command: &str, args: &[&str], use_sudo: bool) -> Result<()> {
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

    let mut child = cmd
        .args(args)
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn()
        .with_context(|| format!("Failed to execute {}", command))?;

    let status = child.wait()?;

    if !status.success() {
        return Err(anyhow!(
            "Command {} failed with status: {}",
            command,
            status
        ));
    }

    Ok(())
}

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

pub fn run_command(command: &str, args: &[&str], with_stdin: bool) -> Result<Output> {
    let mut cmd = Command::new(command);

    match with_stdin {
        true => cmd
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit()),
        false => cmd.stderr(Stdio::piped()),
    };

    let output = cmd
        .args(args)
        .output()
        .with_context(|| format!("Failed to execute {}", command))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow!(
            "Command {} failed: {} {} {} {}",
            command,
            stderr,
            stdout,
            args.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            output.status.code().unwrap_or(-1),
        ));
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

pub fn run_ssh_command(key_path: &str, guest_ip: &str, command: &str) -> Result<()> {
    run_command_with_stdout_inherit(
        "ssh",
        &[
            "-q",
            "-i",
            key_path,
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "UserKnownHostsFile=/dev/null",
            &format!("root@{}", guest_ip),
            command,
        ],
        false,
    )?;
    Ok(())
}

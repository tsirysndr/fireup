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

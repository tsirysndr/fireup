use anyhow::{anyhow, Context, Result};
use std::process::{Command, Output, Stdio};

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

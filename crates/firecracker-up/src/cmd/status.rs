use anyhow::Error;
use owo_colors::OwoColorize;

pub fn status() -> Result<(), Error> {
    if firecracker_process::is_running() {
        println!(
            "Firecracker MicroVM is running. {}",
            "[✓] RUNNING".bright_green()
        );
        return Ok(());
    }

    println!(
        "Firecracker MicroVM is not running. {}",
        "[✗] STOPPED".bright_red()
    );
    Ok(())
}

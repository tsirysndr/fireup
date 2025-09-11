use anyhow::Error;
use owo_colors::OwoColorize;

pub async fn status(name: Option<String>) -> Result<(), Error> {
    match name {
        Some(name) => {
            if firecracker_process::vm_is_running(&name).await? {
                println!(
                    "{} is running. {}",
                    name.cyan(),
                    "[✓] RUNNING".bright_green()
                );
                return Ok(());
            }

            println!(
                "{} is not running. {}",
                name.cyan(),
                "[✗] STOPPED".bright_red()
            );
        }
        None => {
            if firecracker_process::is_running() {
                println!("Firecracker is running. {}", "[✓] RUNNING".bright_green());
                return Ok(());
            }

            println!("Firecracker is not running. {}", "[✗] STOPPED".bright_red());
        }
    }

    Ok(())
}

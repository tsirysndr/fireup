use anyhow::Error;

use crate::{command::run_command, types::VmOptions};

pub fn setup_mosquitto(_config: &VmOptions) -> Result<(), Error> {
    println!("[+] Checking if Mosquitto is installed...");
    if !mosquitto_is_installed()? {
        run_command(
            "apt-get",
            &["install", "-y", "mosquitto", "mosquitto-clients"],
            true,
        )?;
    }

    restart_mosquitto()?;
    println!("[✓] Mosquitto is set up successfully.");
    Ok(())
}

pub fn restart_mosquitto() -> Result<(), Error> {
    println!("[+] Starting mosquitto...");
    run_command("systemctl", &["enable", "mosquitto"], true)?;
    run_command("systemctl", &["restart", "mosquitto"], true)?;
    println!("[✓] Mosquitto started successfully.");
    Ok(())
}

pub fn mosquitto_is_installed() -> Result<bool, Error> {
    let output = run_command("which", &["mosquitto"], false)?;
    Ok(output.status.success())
}

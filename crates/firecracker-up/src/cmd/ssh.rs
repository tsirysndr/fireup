use crate::{command::run_command, config::get_config_dir};
use anyhow::Error;
use firecracker_vm::constants::GUEST_IP;
use glob::glob;

pub fn ssh() -> Result<(), Error> {
    let app_dir = get_config_dir()?;
    let private_key = glob(format!("{}/id_rsa", app_dir).as_str())
        .map_err(|e| Error::msg(format!("Failed to find SSH key: {}", e)))?
        .last()
        .ok_or_else(|| Error::msg("No SSH key file found"))?;
    run_command(
        "ssh",
        &[
            "-i",
            &private_key?.display().to_string(),
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "UserKnownHostsFile=/dev/null",
            &format!("root@{}", GUEST_IP),
        ],
        true,
    )?;
    Ok(())
}

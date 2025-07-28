use anyhow::Error;
use glob::glob;

use crate::{command::run_command, config::get_config_dir};

pub fn ssh() -> Result<(), Error> {
    let app_dir = get_config_dir()?;
    let private_key = glob(format!("{}/*.id_rsa", app_dir).as_str())
        .map_err(|e| Error::msg(format!("Failed to find SSH key: {}", e)))?
        .last()
        .ok_or_else(|| Error::msg("No SSH key file found"))?;
    run_command(
        "ssh",
        &["-i", &private_key?.display().to_string(), "root@172.16.0.2"],
        true,
    )?;
    Ok(())
}

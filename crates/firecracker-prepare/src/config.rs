use anyhow::Error;
use anyhow::{anyhow, Context};
use std::fs;

pub fn get_config_dir() -> Result<String, Error> {
    let app_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("Failed to get home directory"))?
        .join(".fireup");
    fs::create_dir_all(&app_dir)
        .with_context(|| format!("Failed to create app directory: {}", app_dir.display()))?;

    Ok(app_dir.display().to_string())
}

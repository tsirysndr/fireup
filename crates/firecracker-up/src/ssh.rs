use anyhow::Error;
use anyhow::{anyhow, Context, Result};
use std::fs;

pub fn get_private_key_path() -> Result<String, Error> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Failed to get home directory"))?;
    let app_dir = format!("{}/.fireup", home_dir.display());
    let key_name = glob::glob(format!("{}/id_rsa", app_dir).as_str())
        .with_context(|| "Failed to glob ssh key files")?
        .last()
        .ok_or_else(|| anyhow!("No SSH key file found"))?
        .with_context(|| "Failed to get SSH key path")?;
    let key_name = fs::canonicalize(&key_name)
        .with_context(|| {
            format!(
                "Failed to resolve absolute path for SSH key: {}",
                key_name.display()
            )
        })?
        .display()
        .to_string();
    Ok(key_name)
}

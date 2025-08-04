use std::{path::Path, process};

use anyhow::Error;
use firecracker_prepare::Distro;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Vm {
    pub vcpu: Option<u16>,
    pub memory: Option<u16>,
    pub vmlinux: Option<String>,
    pub rootfs: Option<String>,
    pub boot_args: Option<String>,
    pub bridge: Option<String>,
    pub tap: Option<String>,
    pub api_socket: Option<String>,
    pub mac: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FireConfig {
    pub distro: Distro,
    pub vm: Vm,
}

impl Default for FireConfig {
    fn default() -> Self {
        FireConfig {
            distro: Distro::Ubuntu,
            vm: Vm {
                vcpu: Some(num_cpus::get() as u16),
                memory: Some(512),
                vmlinux: None,
                rootfs: None,
                boot_args: None,
                bridge: None,
                tap: None,
                api_socket: None,
                mac: None,
            },
        }
    }
}

pub fn init_config() -> Result<(), Error> {
    if Path::new("fire.toml").exists() {
        println!(
            "Configuration file {} already exists, would you like to overwrite it? (y/n)",
            "'fire.toml'".cyan()
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            process::exit(1);
        }
    }
    let config_path = Path::new("fire.toml");
    let config = FireConfig::default();
    let toml_content = toml::to_string(&config)?;
    std::fs::write(config_path, toml_content)?;
    Ok(())
}

pub fn read_config() -> Result<FireConfig, Error> {
    let config_path = Path::new("fire.toml");
    if !config_path.exists() {
        return Err(Error::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Configuration file not found",
        )));
    }
    let content = std::fs::read_to_string(config_path)?;
    let config: FireConfig = toml::from_str(&content)?;
    Ok(config)
}

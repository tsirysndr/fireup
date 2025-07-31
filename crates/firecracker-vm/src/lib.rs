use anyhow::{anyhow, Context, Result};
use firecracker_prepare::Distro;
use owo_colors::OwoColorize;
use std::fs;

use crate::config::get_config_dir;

mod command;
mod config;
pub mod constants;
mod firecracker;
mod guest;
mod network;

pub fn setup(distro: Distro, vcpu: u16, memory: u16) -> Result<()> {
    let app_dir = get_config_dir().with_context(|| "Failed to get configuration directory")?;

    let logfile = format!("{}/firecracker.log", app_dir);
    fs::File::create(&logfile)
        .with_context(|| format!("Failed to create log file: {}", logfile))?;

    let kernel = glob::glob(format!("{}/vmlinux*", app_dir).as_str())
        .with_context(|| "Failed to glob kernel files")?
        .last()
        .ok_or_else(|| anyhow!("No kernel file found"))?
        .with_context(|| "Failed to get kernel path")?;
    let kernel = fs::canonicalize(&kernel)
        .with_context(|| {
            format!(
                "Failed to resolve absolute path for kernel: {}",
                kernel.display()
            )
        })?
        .display()
        .to_string();

    let ext4_file = match distro {
        Distro::Debian => format!("{}/debian*.ext4", app_dir),
        Distro::Alpine => format!("{}/alpine*.ext4", app_dir),
        Distro::NixOS => format!("{}/nixos*.ext4", app_dir),
        Distro::Ubuntu => format!("{}/ubuntu*.ext4", app_dir),
    };

    let rootfs = glob::glob(&ext4_file)
        .with_context(|| "Failed to glob rootfs files")?
        .last()
        .ok_or_else(|| anyhow!("No rootfs file found"))?
        .with_context(|| "Failed to get rootfs path")?;
    let rootfs = fs::canonicalize(&rootfs)
        .with_context(|| {
            format!(
                "Failed to resolve absolute path for rootfs: {}",
                rootfs.display()
            )
        })?
        .display()
        .to_string();

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
    let arch = command::run_command("uname", &["-m"], false)?.stdout;
    let arch = String::from_utf8_lossy(&arch).trim().to_string();
    network::setup_network()?;
    firecracker::configure(&logfile, &kernel, &rootfs, &arch, vcpu, memory)?;

    if !rootfs.contains("nixos") {
        guest::configure_guest_network(&key_name)?;
    }

    println!("[✓] MicroVM booted and network is configured 🎉");

    println!("SSH into the VM using the following command:");
    println!("{}", "fireup ssh".bright_green());

    Ok(())
}

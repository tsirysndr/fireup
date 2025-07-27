use anyhow::{anyhow, Context, Result};
use owo_colors::OwoColorize;
use std::fs;

use crate::constants::GUEST_IP;

mod command;
mod constants;
mod firecracker;
mod guest;
mod network;

pub fn setup() -> Result<()> {
    let logfile = format!("{}/firecracker.log", std::env::current_dir()?.display());
    fs::File::create(&logfile)
        .with_context(|| format!("Failed to create log file: {}", logfile))?;

    let kernel = glob::glob("vmlinux*")
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

    let rootfs = glob::glob("*.ext4")
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

    let key_name = glob::glob("*.id_rsa")
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
    firecracker::configure(&logfile, &kernel, &rootfs, &arch)?;
    guest::configure_guest_network(&key_name)?;

    println!("[✓] MicroVM booted and network is configured.");

    let key_name = key_name
        .rsplit('/')
        .next()
        .ok_or_else(|| anyhow!("Failed to extract key name from path"))?
        .to_string();

    println!("SSH into the VM using:");
    println!(
        "{}",
        format!("ssh -i ./{} root@{}", key_name, GUEST_IP).bright_green()
    );

    Ok(())
}

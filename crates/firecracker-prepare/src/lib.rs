use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::fs;

pub mod command;
pub mod downloader;
pub mod rootfs;
pub mod ssh;

pub fn prepare() -> Result<()> {
    let arch = command::run_command("uname", &["-m"], false)?.stdout;
    let arch = String::from_utf8_lossy(&arch).trim().to_string();

    println!("[+] Detected architecture: {}", arch.bright_green());

    let (kernel_file, ubuntu_file, ubuntu_version) = downloader::download_files(&arch)?;
    let squashfs_root_dir = "squashfs-root";

    rootfs::extract_squashfs(&ubuntu_file, squashfs_root_dir)?;

    let ssh_key_name = format!("ubuntu-{}.id_rsa", ubuntu_version);
    ssh::generate_and_copy_ssh_key(&ssh_key_name, squashfs_root_dir)?;

    let ext4_file = format!("ubuntu-{}.ext4", ubuntu_version);

    rootfs::create_ext4_filesystem(squashfs_root_dir, &ext4_file)?;

    let kernel_abs = fs::canonicalize(&kernel_file).with_context(|| {
        format!(
            "Failed to resolve absolute path for kernel: {}",
            kernel_file
        )
    })?;
    let ext4_abs = fs::canonicalize(&ext4_file)
        .with_context(|| format!("Failed to resolve absolute path for rootfs: {}", ext4_file))?;
    let ssh_key_abs = fs::canonicalize(&ssh_key_name).with_context(|| {
        format!(
            "Failed to resolve absolute path for SSH key: {}",
            ssh_key_name
        )
    })?;

    println!("[✓] Kernel: {}", kernel_abs.display().bright_green());
    println!("[✓] Rootfs: {}", ext4_abs.display().bright_green());
    println!("[✓] SSH Key: {}", ssh_key_abs.display().bright_green());

    Ok(())
}

use std::fs;

use anyhow::Result;
use owo_colors::OwoColorize;

use crate::command::{run_command, run_command_with_stdout_inherit};

pub mod command;
pub mod config;
pub mod downloader;
pub mod rootfs;
pub mod ssh;

#[derive(Default, Clone)]
pub struct PrepareOptions {
    pub debian: Option<bool>,
    pub alpine: Option<bool>,
    pub ubuntu: Option<bool>,
}

pub fn prepare(options: PrepareOptions) -> Result<()> {
    let arch = command::run_command("uname", &["-m"], false)?.stdout;
    let arch = String::from_utf8_lossy(&arch).trim().to_string();

    println!("[+] Detected architecture: {}", arch.bright_green());

    let (kernel_file, ext4_file, ssh_key_file) =
        match (options.debian, options.alpine, options.ubuntu) {
            (Some(true), _, _) => prepare_debian(&arch)?,
            (_, Some(true), _) => prepare_alpine(&arch)?,
            (_, _, Some(true)) | (_, _, None) => prepare_ubuntu(&arch)?,
            _ => {
                return Err(anyhow::anyhow!("No valid rootfs option provided."));
            }
        };

    println!("[✓] Kernel: {}", kernel_file.bright_green());
    println!("[✓] Rootfs: {}", ext4_file.bright_green());
    println!("[✓] SSH Key: {}", ssh_key_file.bright_green());

    Ok(())
}

pub fn prepare_ubuntu(arch: &str) -> Result<(String, String, String)> {
    let (kernel_file, ubuntu_file, ubuntu_version) = downloader::download_files(arch)?;

    let app_dir = config::get_config_dir()?;
    let squashfs_root_dir = format!("{}/squashfs_root", app_dir);

    rootfs::extract_squashfs(&ubuntu_file, &squashfs_root_dir)?;

    let ssh_key_name = "id_rsa";
    ssh::generate_and_copy_ssh_key(&ssh_key_name, &squashfs_root_dir)?;

    let ext4_file = format!("{}/ubuntu-{}.ext4", app_dir, ubuntu_version);

    if !std::path::Path::new(&ext4_file).exists() {
        rootfs::create_ext4_filesystem(&squashfs_root_dir, &ext4_file, 400)?;
    } else {
        println!(
            "[!] {} already exists, skipping ext4 creation.",
            ext4_file.bright_yellow()
        );
    }

    let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);

    Ok((kernel_file, ext4_file, ssh_key_file))
}

pub fn prepare_debian(arch: &str) -> Result<(String, String, String)> {
    let kernel_file = downloader::download_kernel(arch)?;
    let app_dir = config::get_config_dir()?;
    let debootstrap_dir: &str = &format!("{}/debootstrap", app_dir);

    let arch = match arch {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => arch,
    };

    if !std::path::Path::new(debootstrap_dir).exists() {
        fs::create_dir_all(debootstrap_dir)?;
        run_command_with_stdout_inherit(
            "debootstrap",
            &[
                &format!("--arch={}", arch),
                "stable",
                debootstrap_dir,
                "http://deb.debian.org/debian/",
            ],
            true,
        )?;
    }

    let ssh_key_name = "id_rsa";
    run_command(
        "mkdir",
        &["-p", &format!("{}/root/.ssh", debootstrap_dir)],
        true,
    )?;
    ssh::generate_and_copy_ssh_key(&ssh_key_name, &debootstrap_dir)?;

    if !run_command("chroot", &[debootstrap_dir, "which", "sshd"], true)
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        run_command_with_stdout_inherit("chroot", &[debootstrap_dir, "apt-get", "update"], true)?;
        run_command_with_stdout_inherit(
            "chroot",
            &[
                debootstrap_dir,
                "apt-get",
                "install",
                "-y",
                "openssh-server",
            ],
            true,
        )?;
        run_command(
            "chroot",
            &[debootstrap_dir, "systemctl", "enable", "ssh"],
            true,
        )?;
    }

    let ext4_file = format!("{}/debian-{}.ext4", app_dir, arch);
    if !std::path::Path::new(&ext4_file).exists() {
        rootfs::create_ext4_filesystem(debootstrap_dir, &ext4_file, 600)?;
    }

    let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);

    Ok((kernel_file, ext4_file, ssh_key_file))
}

pub fn prepare_alpine(arch: &str) -> Result<(String, String, String)> {
    let kernel_file = downloader::download_kernel(arch)?;
    unimplemented!("Alpine preparation is not implemented yet.");
}

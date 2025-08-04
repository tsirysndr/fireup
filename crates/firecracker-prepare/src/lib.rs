use std::fs;

use anyhow::Result;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::command::{run_command, run_command_with_stdout_inherit};

pub mod command;
pub mod config;
pub mod downloader;
pub mod rootfs;
pub mod ssh;

const BRIDGE_IP: &str = "172.16.0.1";

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum Distro {
    Debian,
    Alpine,
    Ubuntu,
    NixOS,
}

pub trait RootfsPreparer {
    fn prepare(&self, arch: &str, app_dir: &str) -> Result<(String, String, String)>;
    fn name(&self) -> &'static str;
}

pub struct DebianPreparer;
pub struct AlpinePreparer;
pub struct UbuntuPreparer;
pub struct NixOSPreparer;

impl RootfsPreparer for DebianPreparer {
    fn name(&self) -> &'static str {
        "Debian"
    }

    fn prepare(&self, arch: &str, app_dir: &str) -> Result<(String, String, String)> {
        println!(
            "[+] Preparing {} rootfs for {}...",
            self.name(),
            arch.bright_green()
        );
        let kernel_file = downloader::download_kernel(arch)?;
        let debootstrap_dir = format!("{}/debootstrap", app_dir);

        let arch = match arch {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            _ => arch,
        };

        if !std::path::Path::new(&debootstrap_dir).exists() {
            fs::create_dir_all(&debootstrap_dir)?;
            run_command_with_stdout_inherit(
                "debootstrap",
                &[
                    &format!("--arch={}", arch),
                    "stable",
                    &debootstrap_dir,
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

        if !run_command("chroot", &[&debootstrap_dir, "which", "sshd"], true)
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            run_command_with_stdout_inherit(
                "chroot",
                &[&debootstrap_dir, "apt-get", "update"],
                true,
            )?;
            run_command_with_stdout_inherit(
                "chroot",
                &[
                    &debootstrap_dir,
                    "apt-get",
                    "install",
                    "-y",
                    "openssh-server",
                ],
                true,
            )?;
            run_command(
                "chroot",
                &[&debootstrap_dir, "systemctl", "enable", "ssh"],
                true,
            )?;
        }

        let ext4_file = format!("{}/debian-{}.ext4", app_dir, arch);
        if !std::path::Path::new(&ext4_file).exists() {
            rootfs::create_ext4_filesystem(&debootstrap_dir, &ext4_file, 600)?;
        }

        let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);

        Ok((kernel_file, ext4_file, ssh_key_file))
    }
}

impl RootfsPreparer for AlpinePreparer {
    fn name(&self) -> &'static str {
        "Alpine"
    }

    fn prepare(&self, arch: &str, app_dir: &str) -> Result<(String, String, String)> {
        println!(
            "[+] Preparing {} rootfs for {}...",
            self.name(),
            arch.bright_green()
        );
        let kernel_file = downloader::download_kernel(arch)?;
        let minirootfs = format!("{}/minirootfs", app_dir);
        downloader::download_alpine_rootfs(&minirootfs, arch)?;

        run_command(
            "sh",
            &[
                "-c",
                &format!(
                    "echo 'nameserver {}' >> {}/etc/resolv.conf",
                    BRIDGE_IP, minirootfs
                ),
            ],
            true,
        )?;
        if !run_command("chroot", &[&minirootfs, "which", "sshd"], true)
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            run_command_with_stdout_inherit("chroot", &[&minirootfs, "apk", "update"], true)?;
            run_command_with_stdout_inherit(
                "chroot",
                &[
                    &minirootfs,
                    "apk",
                    "add",
                    "alpine-base",
                    "util-linux",
                    "linux-virt",
                    "haveged",
                    "openssh",
                ],
                true,
            )?;
        }

        run_command_with_stdout_inherit(
            "chroot",
            &[&minirootfs, "rc-update", "add", "haveged"],
            true,
        )?;
        run_command(
            "chroot",
            &[
                &minirootfs,
                "sh",
                "-c",
                "for svc in devfs procfs sysfs; do ln -fs /etc/init.d/$svc /etc/runlevels/boot; done",
            ],
            true,
        )?;
        if !run_command(
            "chroot",
            &[
                &minirootfs,
                "ln",
                "-s",
                "agetty",
                "/etc/init.d/agetty.ttyS0",
            ],
            true,
        )
        .map(|output| output.status.success())
        .unwrap_or(false)
        {
            println!("[!] Failed to create symlink for agetty.ttyS0, please check manually.");
        }
        run_command_with_stdout_inherit(
            "chroot",
            &[&minirootfs, "sh", "-c", "echo ttyS0 > /etc/securetty"],
            true,
        )?;
        run_command(
            "chroot",
            &[&minirootfs, "rc-update", "add", "agetty.ttyS0", "default"],
            true,
        )?;

        run_command("chroot", &[&minirootfs, "rc-update", "add", "sshd"], true)?;
        run_command(
            "chroot",
            &[&minirootfs, "rc-update", "add", "networking", "boot"],
            true,
        )?;
        run_command(
            "chroot",
            &[&minirootfs, "mkdir", "-p", "/root/.ssh", "/etc/network"],
            true,
        )?;

        run_command(
            "chroot",
            &[
                &minirootfs,
                "sh",
                "-c",
                "echo 'auto eth0\niface eth0 inet dhcp' > /etc/network/interfaces",
            ],
            true,
        )?;

        let ssh_key_name = "id_rsa";
        ssh::generate_and_copy_ssh_key(&ssh_key_name, &minirootfs)?;

        let ext4_file = format!("{}/alpine-{}.ext4", app_dir, arch);
        if !std::path::Path::new(&ext4_file).exists() {
            rootfs::create_ext4_filesystem(&minirootfs, &ext4_file, 500)?;
        }

        let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);

        Ok((kernel_file, ext4_file, ssh_key_file))
    }
}

impl RootfsPreparer for UbuntuPreparer {
    fn name(&self) -> &'static str {
        "Ubuntu"
    }

    fn prepare(&self, arch: &str, app_dir: &str) -> Result<(String, String, String)> {
        println!(
            "[+] Preparing {} rootfs for {}...",
            self.name(),
            arch.bright_green()
        );
        let (kernel_file, ubuntu_file, ubuntu_version) = downloader::download_files(arch)?;

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
}

impl RootfsPreparer for NixOSPreparer {
    fn name(&self) -> &'static str {
        "NixOS"
    }

    fn prepare(&self, arch: &str, app_dir: &str) -> Result<(String, String, String)> {
        println!(
            "[+] Preparing {} rootfs for {}...",
            self.name(),
            arch.bright_green()
        );
        let kernel_file = downloader::download_kernel(arch)?;
        let nixos_rootfs = format!("{}/nixosrootfs", app_dir);
        let squashfs_file = format!("{}/nixos-rootfs.squashfs", app_dir);

        downloader::download_nixos_rootfs(arch)?;
        rootfs::extract_squashfs(&squashfs_file, &nixos_rootfs)?;

        let ssh_key_name = "id_rsa";
        ssh::generate_and_copy_ssh_key_nixos(&ssh_key_name, &nixos_rootfs)?;

        let ext4_file = format!("{}/nixos-rootfs.ext4", app_dir);
        if !std::path::Path::new(&ext4_file).exists() {
            rootfs::create_ext4_filesystem(&nixos_rootfs, &ext4_file, 5120)?;
        }

        let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);

        println!(
            "[+] {} rootfs prepared at: {}",
            self.name(),
            nixos_rootfs.bright_green()
        );

        Ok((kernel_file, ext4_file, ssh_key_file))
    }
}

pub fn prepare(distro: Distro) -> Result<()> {
    let arch = run_command("uname", &["-m"], false)?.stdout;
    let arch = String::from_utf8_lossy(&arch).trim().to_string();
    println!("[+] Detected architecture: {}", arch.bright_green());

    let app_dir = config::get_config_dir()?;
    let preparer: Box<dyn RootfsPreparer> = match distro {
        Distro::Debian => Box::new(DebianPreparer),
        Distro::Alpine => Box::new(AlpinePreparer),
        Distro::Ubuntu => Box::new(UbuntuPreparer),
        Distro::NixOS => Box::new(NixOSPreparer),
    };

    let (kernel_file, ext4_file, ssh_key_file) = preparer.prepare(&arch, &app_dir)?;

    println!("[✓] Kernel: {}", kernel_file.bright_green());
    println!("[✓] Rootfs: {}", ext4_file.bright_green());
    println!("[✓] SSH Key: {}", ssh_key_file.bright_green());

    Ok(())
}

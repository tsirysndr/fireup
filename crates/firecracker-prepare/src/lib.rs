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
    Fedora,
    Gentoo,
    Slackware,
    Opensuse,
    OpensuseTumbleweed,
    Almalinux,
    RockyLinux,
    Archlinux,
}

impl ToString for Distro {
    fn to_string(&self) -> String {
        match self {
            Distro::Debian => "debian".to_string(),
            Distro::Alpine => "alpine".to_string(),
            Distro::Ubuntu => "ubuntu".to_string(),
            Distro::NixOS => "nixos".to_string(),
            Distro::Fedora => "fedora".to_string(),
            Distro::Gentoo => "gentoo".to_string(),
            Distro::Slackware => "slackware".to_string(),
            Distro::Opensuse => "opensuse".to_string(),
            Distro::OpensuseTumbleweed => "opensuse-tumbleweed".to_string(),
            Distro::Almalinux => "almalinux".to_string(),
            Distro::RockyLinux => "rockylinux".to_string(),
            Distro::Archlinux => "archlinux".to_string(),
        }
    }
}

pub trait RootfsPreparer {
    fn prepare(&self, arch: &str, app_dir: &str) -> Result<(String, String, String)>;
    fn name(&self) -> &'static str;
}

pub struct DebianPreparer;
pub struct AlpinePreparer;
pub struct UbuntuPreparer;
pub struct NixOSPreparer;
pub struct FedoraPreparer;
pub struct GentooPreparer;
pub struct SlackwarePreparer;
pub struct OpensusePreparer;
pub struct OpensuseTumbleweedPreparer;
pub struct AlmalinuxPreparer;
pub struct RockyLinuxPreparer;
pub struct ArchlinuxPreparer;

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

        run_command(
            "chroot",
            &[
                &debootstrap_dir,
                "sh",
                "-c",
                "apt-get install -y systemd-resolved",
            ],
            true,
        )?;
        run_command(
            "chroot",
            &[
                &debootstrap_dir,
                "systemctl",
                "enable",
                "systemd-networkd",
                "systemd-resolved",
            ],
            true,
        )?;

        const RESOLVED_CONF: &str = include_str!("./config/resolved.conf");
        run_command(
            "chroot",
            &[
                &debootstrap_dir,
                "sh",
                "-c",
                &format!("echo '{}' > /etc/systemd/resolved.conf", RESOLVED_CONF),
            ],
            true,
        )?;

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

        let img_file = format!("{}/debian-rootfs.img", app_dir);
        rootfs::create_overlay_dirs(&debootstrap_dir)?;
        rootfs::add_overlay_init(&debootstrap_dir)?;
        rootfs::create_squashfs(&debootstrap_dir, &img_file)?;

        let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);

        Ok((kernel_file, img_file, ssh_key_file))
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

        let img_file = format!("{}/alpine-rootfs.img", app_dir);
        rootfs::create_squashfs(&minirootfs, &img_file)?;

        let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);

        Ok((kernel_file, img_file, ssh_key_file))
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
        let (kernel_file, ubuntu_file, _ubuntu_version) = downloader::download_files(arch)?;

        let squashfs_root_dir = format!("{}/squashfs_root", app_dir);
        rootfs::extract_squashfs(&ubuntu_file, &squashfs_root_dir)?;

        run_command(
            "chroot",
            &[
                &squashfs_root_dir,
                "systemctl",
                "enable",
                "systemd-networkd",
            ],
            true,
        )?;

        const RESOLVED_CONF: &str = include_str!("./config/resolved.conf");
        run_command(
            "chroot",
            &[
                &squashfs_root_dir,
                "sh",
                "-c",
                &format!("echo '{}' > /etc/systemd/resolved.conf", RESOLVED_CONF),
            ],
            true,
        )?;

        let ssh_key_name = "id_rsa";
        ssh::generate_and_copy_ssh_key(&ssh_key_name, &squashfs_root_dir)?;

        let img_file = format!("{}/ubuntu-rootfs.img", app_dir);
        rootfs::create_overlay_dirs(&squashfs_root_dir)?;
        rootfs::add_overlay_init(&squashfs_root_dir)?;
        rootfs::create_squashfs(&squashfs_root_dir, &img_file)?;

        let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);

        Ok((kernel_file, img_file, ssh_key_file))
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
        let nixos_rootfs = format!("{}/nixos-rootfs", app_dir);
        let squashfs_file = format!("{}/nixos-rootfs.squashfs", app_dir);

        downloader::download_nixos_rootfs(arch)?;
        rootfs::extract_squashfs(&squashfs_file, &nixos_rootfs)?;

        let ssh_key_name = "id_rsa";
        ssh::generate_and_copy_ssh_key_nixos(&ssh_key_name, &nixos_rootfs)?;

        let img_file = format!("{}/nixos-rootfs.img", app_dir);
        rootfs::create_squashfs(&nixos_rootfs, &img_file)?;

        let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);

        println!(
            "[+] {} rootfs prepared at: {}",
            self.name(),
            nixos_rootfs.bright_green()
        );

        Ok((kernel_file, img_file, ssh_key_file))
    }
}

impl RootfsPreparer for FedoraPreparer {
    fn name(&self) -> &'static str {
        "Fedora"
    }

    fn prepare(&self, arch: &str, app_dir: &str) -> Result<(String, String, String)> {
        println!(
            "[+] Preparing {} rootfs for {}...",
            self.name(),
            arch.bright_green()
        );

        let kernel_file = downloader::download_kernel(arch)?;
        let fedora_rootfs = format!("{}/fedora-rootfs", app_dir);
        let squashfs_file = format!("{}/fedora-rootfs.squashfs", app_dir);

        downloader::download_fedora_rootfs(arch)?;
        rootfs::extract_squashfs(&squashfs_file, &fedora_rootfs)?;

        let ssh_key_name = "id_rsa";
        ssh::generate_and_copy_ssh_key(&ssh_key_name, &fedora_rootfs)?;

        let img_file = format!("{}/fedora-rootfs.img", app_dir);
        rootfs::create_squashfs(&fedora_rootfs, &img_file)?;

        let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);

        println!(
            "[+] {} rootfs prepared at: {}",
            self.name(),
            fedora_rootfs.bright_green()
        );

        Ok((kernel_file, img_file, ssh_key_file))
    }
}

impl RootfsPreparer for GentooPreparer {
    fn name(&self) -> &'static str {
        "Gentoo"
    }

    fn prepare(&self, arch: &str, app_dir: &str) -> Result<(String, String, String)> {
        println!(
            "[+] Preparing {} rootfs for {}...",
            self.name(),
            arch.bright_green()
        );

        let kernel_file = downloader::download_kernel(arch)?;
        let gentoo_rootfs = format!("{}/gentoo-rootfs", app_dir);
        let squashfs_file = format!("{}/gentoo-rootfs.squashfs", app_dir);

        downloader::download_gentoo_rootfs(arch)?;
        rootfs::extract_squashfs(&squashfs_file, &gentoo_rootfs)?;

        let ssh_key_name = "id_rsa";
        ssh::generate_and_copy_ssh_key(&ssh_key_name, &gentoo_rootfs)?;

        let img_file = format!("{}/gentoo-rootfs.img", app_dir);
        rootfs::create_squashfs(&gentoo_rootfs, &img_file)?;

        let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);

        Ok((kernel_file, img_file, ssh_key_file))
    }
}

impl RootfsPreparer for SlackwarePreparer {
    fn name(&self) -> &'static str {
        "Slackware"
    }

    fn prepare(&self, arch: &str, app_dir: &str) -> Result<(String, String, String)> {
        println!(
            "[+] Preparing {} rootfs for {}...",
            self.name(),
            arch.bright_green()
        );

        let kernel_file = downloader::download_kernel(arch)?;
        let slackware_rootfs = format!("{}/slackware-rootfs", app_dir);
        let squashfs_file = format!("{}/slackware-rootfs.squashfs", app_dir);

        downloader::download_slackware_rootfs(arch)?;
        rootfs::extract_squashfs(&squashfs_file, &slackware_rootfs)?;

        let ssh_key_name = "id_rsa";
        ssh::generate_and_copy_ssh_key(&ssh_key_name, &slackware_rootfs)?;

        let img_file = format!("{}/slackware-rootfs.img", app_dir);
        rootfs::create_squashfs(&slackware_rootfs, &img_file)?;

        let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);

        Ok((kernel_file, img_file, ssh_key_file))
    }
}

impl RootfsPreparer for OpensusePreparer {
    fn name(&self) -> &'static str {
        "OpenSUSE (Leap)"
    }

    fn prepare(&self, arch: &str, app_dir: &str) -> Result<(String, String, String)> {
        println!(
            "[+] Preparing {} rootfs for {}...",
            self.name(),
            arch.bright_green()
        );

        let kernel_file = downloader::download_kernel(arch)?;
        let opensuse_rootfs = format!("{}/opensuse-rootfs", app_dir);
        let squashfs_file = format!("{}/opensuse-rootfs.squashfs", app_dir);

        downloader::download_opensuse_rootfs(arch)?;
        rootfs::extract_squashfs(&squashfs_file, &opensuse_rootfs)?;

        let ssh_key_name = "id_rsa";
        ssh::generate_and_copy_ssh_key(&ssh_key_name, &opensuse_rootfs)?;

        let img_file = format!("{}/opensuse-rootfs.img", app_dir);
        rootfs::create_squashfs(&opensuse_rootfs, &img_file)?;

        let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);
        Ok((kernel_file, img_file, ssh_key_file))
    }
}

impl RootfsPreparer for AlmalinuxPreparer {
    fn name(&self) -> &'static str {
        "AlmaLinux"
    }

    fn prepare(&self, arch: &str, app_dir: &str) -> Result<(String, String, String)> {
        println!(
            "[+] Preparing {} rootfs for {}...",
            self.name(),
            arch.bright_green()
        );

        let kernel_file = downloader::download_kernel(arch)?;
        let almalinux_rootfs = format!("{}/almalinux-rootfs", app_dir);
        let squashfs_file = format!("{}/almalinux-rootfs.squashfs", app_dir);

        downloader::download_almalinux_rootfs(arch)?;
        rootfs::extract_squashfs(&squashfs_file, &almalinux_rootfs)?;

        let ssh_key_name = "id_rsa";
        ssh::generate_and_copy_ssh_key(&ssh_key_name, &almalinux_rootfs)?;

        let img_file = format!("{}/almalinux-rootfs.img", app_dir);
        rootfs::create_squashfs(&almalinux_rootfs, &img_file)?;

        let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);

        Ok((kernel_file, img_file, ssh_key_file))
    }
}

impl RootfsPreparer for RockyLinuxPreparer {
    fn name(&self) -> &'static str {
        "RockyLinux"
    }

    fn prepare(&self, arch: &str, app_dir: &str) -> Result<(String, String, String)> {
        println!(
            "[+] Preparing {} rootfs for {}...",
            self.name(),
            arch.bright_green()
        );

        let kernel_file = downloader::download_kernel(arch)?;
        let rockylinux_rootfs = format!("{}/rockylinux-rootfs", app_dir);
        let squashfs_file = format!("{}/rockylinux-rootfs.squashfs", app_dir);

        downloader::download_rockylinux_rootfs(arch)?;
        rootfs::extract_squashfs(&squashfs_file, &rockylinux_rootfs)?;

        let ssh_key_name = "id_rsa";
        ssh::generate_and_copy_ssh_key(&ssh_key_name, &rockylinux_rootfs)?;

        let img_file = format!("{}/rockylinux-rootfs.img", app_dir);
        rootfs::create_squashfs(&rockylinux_rootfs, &img_file)?;

        let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);

        Ok((kernel_file, img_file, ssh_key_file))
    }
}

impl RootfsPreparer for ArchlinuxPreparer {
    fn name(&self) -> &'static str {
        "ArchLinux"
    }

    fn prepare(&self, arch: &str, app_dir: &str) -> Result<(String, String, String)> {
        println!(
            "[+] Preparing {} rootfs for {}...",
            self.name(),
            arch.bright_green()
        );

        let kernel_file = downloader::download_kernel(arch)?;
        let archlinux_rootfs = format!("{}/archlinux-rootfs", app_dir);
        let squashfs_file = format!("{}/archlinux-rootfs.squashfs", app_dir);

        downloader::download_archlinux_rootfs(arch)?;
        rootfs::extract_squashfs(&squashfs_file, &archlinux_rootfs)?;

        let ssh_key_name = "id_rsa";
        ssh::generate_and_copy_ssh_key(&ssh_key_name, &archlinux_rootfs)?;

        let img_file = format!("{}/archlinux-rootfs.img", app_dir);
        rootfs::create_squashfs(&archlinux_rootfs, &img_file)?;

        let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);

        Ok((kernel_file, img_file, ssh_key_file))
    }
}

impl RootfsPreparer for OpensuseTumbleweedPreparer {
    fn name(&self) -> &'static str {
        "OpenSUSE (Tumbleweed)"
    }

    fn prepare(&self, arch: &str, app_dir: &str) -> Result<(String, String, String)> {
        println!(
            "[+] Preparing {} rootfs for {}...",
            self.name(),
            arch.bright_green()
        );

        let kernel_file = downloader::download_kernel(arch)?;
        let opensuse_rootfs = format!("{}/opensuse-tumbleweed-rootfs", app_dir);
        let squashfs_file = format!("{}/opensuse-tumbleweed-rootfs.squashfs", app_dir);

        downloader::download_opensuse_tumbleweed_rootfs(arch)?;
        rootfs::extract_squashfs(&squashfs_file, &opensuse_rootfs)?;

        let ssh_key_name = "id_rsa";
        ssh::generate_and_copy_ssh_key(&ssh_key_name, &opensuse_rootfs)?;

        let img_file = format!("{}/opensuse-tumbleweed-rootfs.img", app_dir);
        rootfs::create_squashfs(&opensuse_rootfs, &img_file)?;

        let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);
        Ok((kernel_file, img_file, ssh_key_file))
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
        Distro::Fedora => Box::new(FedoraPreparer),
        Distro::Gentoo => Box::new(GentooPreparer),
        Distro::Slackware => Box::new(SlackwarePreparer),
        Distro::Opensuse => Box::new(OpensusePreparer),
        Distro::OpensuseTumbleweed => Box::new(OpensuseTumbleweedPreparer),
        Distro::Almalinux => Box::new(AlmalinuxPreparer),
        Distro::RockyLinux => Box::new(RockyLinuxPreparer),
        Distro::Archlinux => Box::new(ArchlinuxPreparer),
    };

    let (kernel_file, img_file, ssh_key_file) = preparer.prepare(&arch, &app_dir)?;

    println!("[✓] Kernel: {}", kernel_file.bright_green());
    println!("[✓] Rootfs: {}", img_file.bright_green());
    println!("[✓] SSH Key: {}", ssh_key_file.bright_green());

    Ok(())
}

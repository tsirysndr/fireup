use anyhow::{anyhow, Context, Result};
use firecracker_prepare::Distro;
use firecracker_state::{entity::virtual_machine::VirtualMachine, repo};
use owo_colors::OwoColorize;
use std::fs;

use crate::{config::get_config_dir, types::VmOptions};

mod command;
mod config;
pub mod constants;
mod coredns;
mod dhcpd;
mod firecracker;
mod guest;
pub mod mac;
mod mosquitto;
mod mqttc;
mod network;
pub mod types;

pub async fn setup(options: &VmOptions, pid: u32, vm_id: Option<String>) -> Result<()> {
    let distro: Distro = options.clone().into();
    let app_dir = get_config_dir().with_context(|| "Failed to get configuration directory")?;

    let name = options
        .api_socket
        .split('/')
        .last()
        .ok_or_else(|| anyhow!("Failed to extract VM name from API socket path"))?
        .replace("firecracker-", "")
        .replace(".sock", "")
        .to_string();
    let name = match name.is_empty() {
        true => names::Generator::default().next().unwrap(),
        false => name,
    };

    fs::create_dir_all(format!("{}/logs", app_dir))
        .with_context(|| format!("Failed to create logs directory: {}", app_dir))?;

    let logfile = format!("{}/logs/firecracker-{}.log", app_dir, name);
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

    // readonly rootfs (squashfs)
    let img_file = match distro {
        Distro::Debian => format!("{}/debian-rootfs.img", app_dir),
        Distro::Alpine => format!("{}/alpine-rootfs.img", app_dir),
        Distro::NixOS => format!("{}/nixos-rootfs.img", app_dir),
        Distro::Ubuntu => format!("{}/ubuntu-rootfs.img", app_dir),
        Distro::Fedora => format!("{}/fedora-rootfs.img", app_dir),
        Distro::Gentoo => format!("{}/gentoo-rootfs.img", app_dir),
        Distro::Slackware => format!("{}/slackware-rootfs.img", app_dir),
        Distro::Opensuse => format!("{}/opensuse-rootfs.img", app_dir),
        Distro::OpensuseTumbleweed => format!("{}/opensuse-tumbleweed-rootfs.img", app_dir),
        Distro::Almalinux => format!("{}/almalinux-rootfs.img", app_dir),
        Distro::RockyLinux => format!("{}/rockylinux-rootfs.img", app_dir),
        Distro::Archlinux => format!("{}/archlinux-rootfs.img", app_dir),
    };

    let rootfs = fs::canonicalize(&img_file)
        .with_context(|| format!("Failed to resolve absolute path for rootfs: {}", img_file))?
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
    network::setup_network(options)?;
    mosquitto::setup_mosquitto(options)?;
    coredns::setup_coredns(options)?;
    dhcpd::setup_kea_dhcp(options)?;

    firecracker::configure(&logfile, &kernel, &rootfs, &arch, &options)?;

    if distro != Distro::NixOS {
        let guest_ip = format!("{}.firecracker", name);
        guest::configure_guest_network(&key_name, &guest_ip)?;
    }
    let pool = firecracker_state::create_connection_pool().await?;

    let ip_file = format!("/tmp/firecracker-{}.ip", name);

    // loop until the IP file is created
    let mut attempts = 0;
    while attempts < 30 {
        println!("[*] Waiting for VM to obtain an IP address...");
        if fs::metadata(&ip_file).is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
        attempts += 1;
    }

    let ip_addr = fs::read_to_string(&ip_file)
        .with_context(|| format!("Failed to read IP address from file: {}", ip_file))?
        .trim()
        .to_string();

    fs::remove_file(&ip_file)
        .with_context(|| format!("Failed to remove IP address file: {}", ip_file))?;

    let project_dir = match fs::metadata("fire.toml").is_ok() {
        true => Some(std::env::current_dir()?.display().to_string()),
        false => None,
    };

    let kernel = match &options.vmlinux {
        Some(path) => path.clone(),
        None => kernel.into(),
    };

    let kernel = fs::canonicalize(&kernel)
        .with_context(|| format!("Failed to canonicalize kernel path: {}", kernel))?
        .display()
        .to_string();

    match vm_id {
        Some(id) => {
            repo::virtual_machine::update(
                &pool,
                &id,
                VirtualMachine {
                    vcpu: options.vcpu,
                    memory: options.memory,
                    api_socket: options.api_socket.clone(),
                    bridge: options.bridge.clone(),
                    tap: options.tap.clone(),
                    mac_address: options.mac_address.clone(),
                    name: name.clone(),
                    pid: Some(pid),
                    distro: distro.to_string(),
                    ip_address: Some(ip_addr.clone()),
                    status: "RUNNING".into(),
                    project_dir,
                    vmlinux: Some(kernel),
                    rootfs: Some(rootfs),
                    bootargs: options.bootargs.clone(),
                    ..Default::default()
                },
            )
            .await?;
        }
        None => {
            repo::virtual_machine::create(
                &pool,
                VirtualMachine {
                    vcpu: options.vcpu,
                    memory: options.memory,
                    api_socket: options.api_socket.clone(),
                    bridge: options.bridge.clone(),
                    tap: options.tap.clone(),
                    mac_address: options.mac_address.clone(),
                    name: name.clone(),
                    pid: Some(pid),
                    distro: distro.to_string(),
                    ip_address: Some(ip_addr.clone()),
                    status: "RUNNING".into(),
                    project_dir,
                    vmlinux: Some(kernel),
                    rootfs: Some(rootfs),
                    bootargs: options.bootargs.clone(),
                    ..Default::default()
                },
            )
            .await?;
        }
    }

    println!("[✓] MicroVM booted and network is configured 🎉");

    println!("SSH into the VM using the following command:");
    println!("{} {}", "fireup ssh".bright_green(), name.bright_green());

    Ok(())
}

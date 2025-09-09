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
mod firecracker;
mod guest;
pub mod mac;
mod network;
mod nextdhcp;
pub mod types;

pub async fn setup(options: &VmOptions, pid: u32) -> Result<()> {
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
    network::setup_network(options)?;
    coredns::setup_coredns(options)?;
    nextdhcp::setup_nextdhcp(options)?;

    firecracker::configure(&logfile, &kernel, &rootfs, &arch, &options, distro)?;

    if distro != Distro::NixOS {
        let guest_ip = format!("{}.firecracker", name);
        guest::configure_guest_network(&key_name, &guest_ip)?;
    }
    let pool = firecracker_state::create_connection_pool().await?;
    let distro = match distro {
        Distro::Debian => "debian".into(),
        Distro::Alpine => "alpine".into(),
        Distro::NixOS => "nixos".into(),
        Distro::Ubuntu => "ubuntu".into(),
    };
    repo::virtual_machine::create(
        pool,
        VirtualMachine {
            vcpu: options.vcpu,
            memory: options.memory,
            api_socket: options.api_socket.clone(),
            bridge: options.bridge.clone(),
            tap: options.tap.clone(),
            mac_address: options.mac_address.clone(),
            name: name.clone(),
            pid: Some(pid),
            distro,
            ..Default::default()
        },
    )
    .await?;

    println!("[✓] MicroVM booted and network is configured 🎉");

    println!("SSH into the VM using the following command:");
    println!("{} {}", "fireup ssh".bright_green(), name.bright_green());

    Ok(())
}

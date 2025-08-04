use crate::types::VmOptions;
use anyhow::Result;
use firecracker_prepare::Distro;
use serde_json::json;
use std::thread::sleep;
use std::time::Duration;

use crate::command::run_command;

const NIXOS_BOOT_ARGS: &str = "init=/nix/store/w1yqjd8sswh8zj9sz2v76dpw3llzkg5k-nixos-system-nixos-firecracker-25.05.802216.55d1f923c480/init root=/dev/vda ro console=ttyS0 reboot=k panic=1 ip=dhcp";

pub fn configure(
    logfile: &str,
    kernel: &str,
    rootfs: &str,
    arch: &str,
    options: &VmOptions,
    distro: Distro,
) -> Result<()> {
    configure_logger(logfile, options)?;
    setup_boot_source(kernel, arch, distro == Distro::NixOS, &options)?;
    setup_rootfs(rootfs, options)?;
    setup_network_interface(options)?;
    setup_vcpu_and_memory(options.vcpu, options.memory, &options.api_socket)?;

    // Wait before starting instance
    sleep(Duration::from_millis(15));

    start_microvm(options)?;

    // Wait for VM to boot
    sleep(Duration::from_secs(2));
    Ok(())
}

fn configure_logger(logfile: &str, options: &VmOptions) -> Result<()> {
    println!("[+] Configuring logger...");
    let payload = json!({
        "log_path": logfile,
        "level": "Debug",
        "show_level": true,
        "show_log_origin": true
    });
    run_command(
        "curl",
        &[
            "-s",
            "-X",
            "PUT",
            "--unix-socket",
            &options.api_socket,
            "--data",
            &payload.to_string(),
            "http://localhost/logger",
        ],
        true,
    )?;
    Ok(())
}

fn setup_boot_source(kernel: &str, arch: &str, is_nixos: bool, options: &VmOptions) -> Result<()> {
    println!("[+] Setting boot source...");
    let mut boot_args = "console=ttyS0 reboot=k panic=1 pci=off ip=dhcp".to_string();
    if arch == "aarch64" {
        boot_args = format!("keep_bootcon {}", boot_args);
    }

    if is_nixos {
        boot_args = NIXOS_BOOT_ARGS.into();
    }

    if let Some(args) = &options.bootargs {
        boot_args = args.clone();
    }

    let payload = json!({
        "kernel_image_path": match &options.vmlinux {
            Some(path) => path.clone(),
            None => kernel.into(),
        },
        "boot_args": boot_args
    });
    println!("{}", payload.to_string());
    run_command(
        "curl",
        &[
            "-s",
            "-X",
            "PUT",
            "--unix-socket",
            &options.api_socket,
            "--data",
            &payload.to_string(),
            "http://localhost/boot-source",
        ],
        true,
    )?;
    Ok(())
}

fn setup_rootfs(rootfs: &str, options: &VmOptions) -> Result<()> {
    println!("[+] Setting rootfs...");
    let payload = json!({
        "drive_id": "rootfs",
        "path_on_host": rootfs,
        "is_root_device": true,
        "is_read_only": false
    });
    run_command(
        "curl",
        &[
            "-s",
            "-X",
            "PUT",
            "--unix-socket",
            &options.api_socket,
            "--data",
            &payload.to_string(),
            "http://localhost/drives/rootfs",
        ],
        true,
    )?;
    Ok(())
}

fn setup_network_interface(options: &VmOptions) -> Result<()> {
    println!("[+] Setting network interface...");
    let iface = "eth0";
    let payload = json!({
        "iface_id": iface,
        "guest_mac": &options.mac_address,
        "host_dev_name": &options.tap
    });

    println!("{}", payload.to_string());
    run_command(
        "curl",
        &[
            "-s",
            "-X",
            "PUT",
            "--unix-socket",
            &options.api_socket,
            "--data",
            &payload.to_string(),
            &format!("http://localhost/network-interfaces/{}", iface),
        ],
        true,
    )?;
    Ok(())
}

fn start_microvm(options: &VmOptions) -> Result<()> {
    println!("[+] Starting microVM...");
    let payload = json!({
        "action_type": "InstanceStart"
    });
    run_command(
        "curl",
        &[
            "-s",
            "-X",
            "PUT",
            "--unix-socket",
            &options.api_socket,
            "--data",
            &payload.to_string(),
            "http://localhost/actions",
        ],
        true,
    )?;
    Ok(())
}

fn setup_vcpu_and_memory(n: u16, memory: u16, api_socket: &str) -> Result<()> {
    println!("[+] Setting vCPU and memory...");
    let payload = json!({
        "vcpu_count": n,
        "mem_size_mib": memory,
        "smt": false,
    });
    println!("{}", payload.to_string());
    run_command(
        "curl",
        &[
            "-s",
            "-X",
            "PUT",
            "--unix-socket",
            api_socket,
            "--data",
            &payload.to_string(),
            "http://localhost/machine-config",
        ],
        true,
    )?;
    Ok(())
}

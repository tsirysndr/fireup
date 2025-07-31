use crate::constants::{API_SOCKET, FC_MAC, TAP_DEV};
use anyhow::Result;
use serde_json::json;
use std::thread::sleep;
use std::time::Duration;

use crate::command::run_command;

const NIXOS_BOOT_ARGS: &str = "init=/nix/store/pq529c6dd6x5vaxak4vpyxrv17ydvnwr-nixos-system-nixos-firecracker-25.05.802216.55d1f923c480/init root=/dev/vda ro console=ttyS0 reboot=k panic=1";

pub fn configure(logfile: &str, kernel: &str, rootfs: &str, arch: &str) -> Result<()> {
    configure_logger(logfile)?;
    setup_boot_source(kernel, arch, rootfs.contains("nixos"))?;
    setup_rootfs(rootfs)?;
    setup_network_interface()?;
    setup_vcpu_and_memory(
        num_cpus::get(),
        if rootfs.contains("nixos") { 2048 } else { 512 },
    )?;

    // Wait before starting instance
    sleep(Duration::from_millis(15));

    start_microvm()?;

    // Wait for VM to boot
    sleep(Duration::from_secs(2));
    Ok(())
}

fn configure_logger(logfile: &str) -> Result<()> {
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
            API_SOCKET,
            "--data",
            &payload.to_string(),
            "http://localhost/logger",
        ],
        true,
    )?;
    Ok(())
}

fn setup_boot_source(kernel: &str, arch: &str, is_nixos: bool) -> Result<()> {
    println!("[+] Setting boot source...");
    let mut boot_args = "console=ttyS0 reboot=k panic=1 pci=off".to_string();
    if arch == "aarch64" {
        boot_args = format!("keep_bootcon {}", boot_args);
    }

    if is_nixos {
        boot_args = NIXOS_BOOT_ARGS.into();
    }

    let payload = json!({
        "kernel_image_path": kernel,
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
            API_SOCKET,
            "--data",
            &payload.to_string(),
            "http://localhost/boot-source",
        ],
        true,
    )?;
    Ok(())
}

fn setup_rootfs(rootfs: &str) -> Result<()> {
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
            API_SOCKET,
            "--data",
            &payload.to_string(),
            "http://localhost/drives/rootfs",
        ],
        true,
    )?;
    Ok(())
}

fn setup_network_interface() -> Result<()> {
    println!("[+] Setting network interface...");
    let iface = "eth0";
    let payload = json!({
        "iface_id": iface,
        "guest_mac": FC_MAC,
        "host_dev_name": TAP_DEV
    });

    println!("{}", payload.to_string());
    run_command(
        "curl",
        &[
            "-s",
            "-X",
            "PUT",
            "--unix-socket",
            API_SOCKET,
            "--data",
            &payload.to_string(),
            &format!("http://localhost/network-interfaces/{}", iface),
        ],
        true,
    )?;
    Ok(())
}

fn start_microvm() -> Result<()> {
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
            API_SOCKET,
            "--data",
            &payload.to_string(),
            "http://localhost/actions",
        ],
        true,
    )?;
    Ok(())
}

fn setup_vcpu_and_memory(n: usize, memory: usize) -> Result<()> {
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
            API_SOCKET,
            "--data",
            &payload.to_string(),
            "http://localhost/machine-config",
        ],
        true,
    )?;
    Ok(())
}

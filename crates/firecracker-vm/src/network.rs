use crate::{
    constants::{BRIDGE_IP, MASK_SHORT},
    types::VmOptions,
};
use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use crate::command::run_command;

fn check_tap_exists(config: &VmOptions) -> bool {
    run_command("ip", &["link", "show", &config.tap], false)
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn check_bridge_exists(config: &VmOptions) -> bool {
    run_command("ip", &["link", "show", &config.bridge], false)
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn create_new_tap(config: &VmOptions) -> Result<()> {
    run_command(
        "ip",
        &["tuntap", "add", "dev", &config.tap, "mode", "tap"],
        true,
    )?;
    run_command("ip", &["link", "set", "dev", &config.tap, "up"], true)?;
    run_command(
        "ip",
        &["link", "set", &config.tap, "master", &config.bridge],
        true,
    )?;
    Ok(())
}

pub fn setup_network(config: &VmOptions) -> Result<()> {
    if check_tap_exists(config) {
        run_command("ip", &["addr", "flush", "dev", &config.tap], true)?;
    }

    if check_tap_exists(config) && check_bridge_exists(config) {
        println!("[✓] Network already configured. Skipping setup.");
        return Ok(());
    }

    if !check_bridge_exists(config) {
        println!("[+] Configuring {}...", config.bridge);
        run_command(
            "ip",
            &["link", "add", "name", &config.bridge, "type", "bridge"],
            true,
        )?;
        run_command("ip", &["link", "set", &config.bridge, "up"], true)?;
        run_command(
            "ip",
            &[
                "addr",
                "add",
                &format!("{}{}", BRIDGE_IP, MASK_SHORT),
                "dev",
                &config.bridge,
            ],
            true,
        )?;
    }

    if !check_tap_exists(config) {
        println!("[+] Configuring {}...", &config.tap);
        create_new_tap(config)?;
    }

    let ip_forward = run_command("cat", &["/proc/sys/net/ipv4/ip_forward"], false)?.stdout;
    if String::from_utf8_lossy(&ip_forward).trim() != "1" {
        println!("[+] Enabling IP forwarding...");
        run_command("sysctl", &["-w", "net.ipv4.ip_forward=1"], true)?;
    }

    let output = run_command("ip", &["-j", "route", "list", "default"], false)?;
    let json: Value =
        serde_json::from_slice(&output.stdout).with_context(|| "Failed to parse route JSON")?;
    let host_iface = json[0]["dev"]
        .as_str()
        .ok_or_else(|| anyhow!("Failed to get host interface"))?;

    println!("[+] Setting up NAT on {}...", host_iface);

    let rule_exists = run_command(
        "iptables",
        &[
            "-t",
            "nat",
            "-C",
            "POSTROUTING",
            "-o",
            host_iface,
            "-j",
            "MASQUERADE",
        ],
        true,
    )
    .map(|output| output.status.success())
    .unwrap_or(false);

    if !rule_exists {
        run_command(
            "iptables",
            &[
                "-t",
                "nat",
                "-A",
                "POSTROUTING",
                "-o",
                host_iface,
                "-j",
                "MASQUERADE",
            ],
            true,
        )?;
    }

    run_command("iptables", &["-P", "FORWARD", "ACCEPT"], true)?;

    Ok(())
}

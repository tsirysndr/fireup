use crate::{
    constants::{BRIDGE_DEV, BRIDGE_IP, MASK_SHORT, TAP_DEV},
    dnsmasq::setup_dnsmasq,
};
use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use crate::command::run_command;

fn check_tap_exists() -> bool {
    run_command("ip", &["link", "show", TAP_DEV], false)
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn check_bridge_exists() -> bool {
    run_command("ip", &["link", "show", BRIDGE_DEV], false)
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn setup_network() -> Result<()> {
    if check_tap_exists() {
        run_command("ip", &["addr", "flush", "dev", TAP_DEV], true)?;
    }

    if check_tap_exists() && check_bridge_exists() {
        println!("[✓] Network already configured. Skipping setup.");
        return Ok(());
    }

    if !check_tap_exists() {
        println!("[+] Configuring {}...", TAP_DEV);
        run_command(
            "ip",
            &["tuntap", "add", "dev", TAP_DEV, "mode", "tap"],
            true,
        )?;
        run_command("ip", &["link", "set", "dev", TAP_DEV, "up"], true)?;
    }

    if !check_bridge_exists() {
        println!("[+] Configuring {}...", BRIDGE_DEV);
        run_command(
            "ip",
            &["link", "add", "name", BRIDGE_DEV, "type", "bridge"],
            true,
        )?;
        run_command("ip", &["link", "set", BRIDGE_DEV, "up"], true)?;
        run_command("ip", &["link", "set", TAP_DEV, "master", BRIDGE_DEV], true)?;
        run_command(
            "ip",
            &[
                "addr",
                "add",
                &format!("{}{}", BRIDGE_IP, MASK_SHORT),
                "dev",
                BRIDGE_DEV,
            ],
            true,
        )?;
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

    setup_dnsmasq()?;

    Ok(())
}

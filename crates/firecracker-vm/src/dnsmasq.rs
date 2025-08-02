use std::process;

use anyhow::Error;

use crate::{
    command::run_command,
    constants::{BRIDGE_DEV, BRIDGE_IP, FC_MAC},
};

pub const DNSMASQ_CONFIG_PATH: &str = "/etc/dnsmasq.d/firecracker.conf";

pub fn setup_dnsmasq() -> Result<(), Error> {
    println!("[+] Checking if DNSMasq is installed...");
    if !dnsmasq_is_installed()? {
        println!("[✗] DNSMasq is not installed. Please install it first.");
        process::exit(1);
    }

    if std::path::Path::new(DNSMASQ_CONFIG_PATH).exists() {
        println!("[✓] DNSMasq configuration already exists. Skipping setup.");
        return Ok(());
    }
    println!("[+] Setting up DNSMasq configuration...");
    run_command("mkdir", &["-p", "/etc/dnsmasq.d"], true)?;

    let dnsmasq_config: &str = &format!(
        r#"
interface={}
bind-interfaces
domain=firecracker.local
dhcp-option=option:router,{}
dhcp-option=option:dns-server,{}
dhcp-range=172.16.0.2,172.16.0.150,12h
dhcp-host={},vm0
server=8.8.8.8
server=8.8.4.4
server=1.1.1.1
"#,
        BRIDGE_DEV, BRIDGE_IP, BRIDGE_IP, FC_MAC
    );

    run_command(
        "sh",
        &[
            "-c",
            &format!("echo '{}' > {}", dnsmasq_config, DNSMASQ_CONFIG_PATH),
        ],
        true,
    )?;

    restart_dnsmasq()?;

    Ok(())
}

pub fn restart_dnsmasq() -> Result<(), Error> {
    println!("[+] Starting DNSMasq...");
    run_command("systemctl", &["enable", "dnsmasq"], true)?;
    run_command("systemctl", &["restart", "dnsmasq"], true)?;
    println!("[✓] DNSMasq started successfully.");
    Ok(())
}

pub fn dnsmasq_is_installed() -> Result<bool, Error> {
    let output = run_command("which", &["dnsmasq"], false)?;
    Ok(output.status.success())
}

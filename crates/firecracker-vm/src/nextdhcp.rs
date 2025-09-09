use std::process;

use anyhow::Error;

use crate::{command::run_command, types::VmOptions};

pub const NEXTDHCP_CONFIG_PATH: &str = "/etc/nextdhcp/Dhcpfile";
pub const NEXTDHCP_SERVICE_TEMPLATE: &str = include_str!("./systemd/nextdhcp.service");

pub fn setup_nextdhcp(config: &VmOptions) -> Result<(), Error> {
    println!("[+] Checking if NextDHCP is installed...");
    if !nextdhcp_is_installed()? {
        // TODO: install it automatically
        println!("[✗] NextDHCP is not installed. Please install it first to /usr/sbin.");
        process::exit(1);
    }

    let nextdhcp_config: &str = &format!(
        r#"
172.16.0.1/24 {{
  lease 1h

  static {} 172.16.0.2

  range 172.16.0.2 172.16.0.150

  option {{
    router 172.16.0.1
    nameserver 172.16.0.1
    }}
  }}
"#,
        &config.mac_address
    );

    run_command(
        "sh",
        &[
            "-c",
            &format!("echo '{}' > {}", nextdhcp_config, NEXTDHCP_CONFIG_PATH),
        ],
        true,
    )?;

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "echo '{}' > /etc/systemd/system/nextdhcp.service",
                NEXTDHCP_SERVICE_TEMPLATE
            ),
        ],
        true,
    )?;
    restart_nextdhcp()?;

    Ok(())
}

pub fn restart_nextdhcp() -> Result<(), Error> {
    println!("[+] Starting nextdhcp...");

    run_command("systemctl", &["enable", "nextdhcp"], true)?;
    run_command("systemctl", &["stop", "nextdhcp"], true)?;
    run_command("rm", &["-f", "/etc/nextdhcp/172.16.0.1.db"], true)?;
    run_command("systemctl", &["start", "nextdhcp"], true)?;
    println!("[✓] Nextdhcp started successfully.");
    Ok(())
}

pub fn nextdhcp_is_installed() -> Result<bool, Error> {
    let output = run_command("which", &["nextdhcp"], false)?;
    Ok(output.status.success())
}

use std::process;

use anyhow::Error;

use crate::{command::run_command, types::VmOptions};

pub const COREDNS_CONFIG_PATH: &str = "/etc/coredns/Corefile";
pub const COREDNS_SERVICE_TEMPLATE: &str = include_str!("./systemd/coredns.service");

pub fn setup_coredns(config: &VmOptions) -> Result<(), Error> {
    println!("[+] Checking if CoreDNS is installed...");
    if !coredns_is_installed()? {
        // TODO: install it automatically
        println!("[✗] CoreDNS is not installed. Please install it first to /usr/sbin.");
        process::exit(1);
    }

    let name = config
        .api_socket
        .split('/')
        .last()
        .ok_or_else(|| anyhow::anyhow!("Failed to extract VM name from API socket path"))?
        .replace("firecracker-", "")
        .replace(".sock", "");

    let hosts = vec![format!("172.16.0.2 {}.firecracker", name)];

    let hosts = hosts.join("\n      ");

    let coredns_config: &str = &format!(
        r#"
  firecracker:53 {{
    hosts {{
      172.16.0.1 br.firecracker
      {}
      fallthrough
    }}

    loadbalance
  }}

  ts.net:53 {{
    # Forward non-internal queries (e.g., to Google DNS)
    forward . 100.100.100.100
    # Log and errors for debugging
    log
    errors
    health
  }}

  .:53 {{
    # Forward non-internal queries (e.g., to Google DNS)
    forward . 8.8.8.8 8.8.4.4 1.1.1.1 1.0.0.1 {{
      max_fails 3
      expire 10s
      health_check 5s
      policy round_robin
      except ts.net
    }}
    # Log and errors for debugging
    log
    errors
    health
  }}
  "#,
        hosts
    );

    run_command(
        "sh",
        &[
            "-c",
            &format!("echo '{}' > {}", coredns_config, COREDNS_CONFIG_PATH),
        ],
        true,
    )?;

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "echo '{}' > /etc/systemd/system/coredns.service",
                COREDNS_SERVICE_TEMPLATE
            ),
        ],
        true,
    )?;
    restart_coredns()?;

    Ok(())
}

pub fn restart_coredns() -> Result<(), Error> {
    println!("[+] Starting CoreDNS...");
    run_command("systemctl", &["enable", "coredns"], true)?;
    run_command("systemctl", &["restart", "coredns"], true)?;
    println!("[✓] CoreDNS started successfully.");
    Ok(())
}

pub fn coredns_is_installed() -> Result<bool, Error> {
    let output = run_command("which", &["coredns"], false)?;
    Ok(output.status.success())
}

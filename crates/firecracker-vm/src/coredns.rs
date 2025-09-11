use std::{process, thread};

use anyhow::{Context, Error};
use firecracker_state::repo;

use crate::{command::run_command, mqttc, types::VmOptions};

pub const COREDNS_CONFIG_PATH: &str = "/etc/coredns/Corefile";
pub const COREDNS_SERVICE_TEMPLATE: &str = include_str!("./systemd/coredns.service");

pub fn setup_coredns(config: &VmOptions) -> Result<(), Error> {
    let api_socket = config.api_socket.clone();
    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        match runtime.block_on(async {
            println!("[+] Checking if CoreDNS is installed...");
            if !coredns_is_installed()? {
                // TODO: install it automatically
                println!("[✗] CoreDNS is not installed. Please install it first to /usr/sbin.");
                process::exit(1);
            }

            let message = mqttc::wait_for_mqtt_message("REQUEST").await?;
            let ip_addr = message
                .split_whitespace()
                .nth(2)
                .ok_or_else(|| anyhow::anyhow!("Failed to extract IP address from MQTT message"))?;

            let name = api_socket
                .split('/')
                .last()
                .ok_or_else(|| anyhow::anyhow!("Failed to extract VM name from API socket path"))?
                .replace("firecracker-", "")
                .replace(".sock", "");

            std::fs::write(format!("/tmp/firecracker-{}.ip", name), ip_addr)
                .with_context(|| "Failed to write IP address to file")?;

            let pool = firecracker_state::create_connection_pool().await?;
            let vms = repo::virtual_machine::all(&pool).await?;
            let mut hosts = vms
                .into_iter()
                .filter(|vm| vm.ip_address.is_some() && vm.name != name)
                .map(|vm| format!("{} {}.firecracker", vm.ip_address.unwrap(), vm.name))
                .collect::<Vec<String>>();

            hosts.extend(vec![format!("{} {}.firecracker", ip_addr, name)]);

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
    # Forward non-internal queries (e.g., to Tailscale DNS)
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

            Ok::<(), Error>(())
        }) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("[✗] Error setting up CoreDNS: {}", e);
                process::exit(1);
            }
        }
        Ok::<(), Error>(())
    });

    Ok(())
}

pub fn restart_coredns() -> Result<(), Error> {
    println!("[+] Starting CoreDNS...");
    run_command("systemctl", &["enable", "coredns"], true)?;
    run_command("systemctl", &["daemon-reload"], true)?;
    run_command("systemctl", &["restart", "coredns"], true)?;
    println!("[✓] CoreDNS started successfully.");
    Ok(())
}

pub fn coredns_is_installed() -> Result<bool, Error> {
    let output = run_command("which", &["coredns"], false)?;
    Ok(output.status.success())
}

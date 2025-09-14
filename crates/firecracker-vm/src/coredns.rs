use std::{process, thread};

use anyhow::{Context, Error};

use crate::{command::run_command, mqttc, types::VmOptions};

pub const COREDNS_CONFIG_PATH: &str = "/etc/coredns/Corefile";
pub const COREDNS_SERVICE_TEMPLATE: &str = include_str!("./systemd/coredns.service");

pub fn setup_coredns(config: &VmOptions) -> Result<(), Error> {
    let api_socket = config.api_socket.clone();
    if !coredns_is_installed()? {
        println!("[✗] CoreDNS is not installed. Please install it first to /usr/sbin.");
        process::exit(1);
    }

    if !etcd_is_installed()? {
        println!("[+] Installing etcd...");
        run_command(
            "apt-get",
            &["install", "-y", "etcd-server", "etcd-client"],
            true,
        )?;
    }

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "echo '{}' > {}",
                include_str!("./coredns/Corefile"),
                COREDNS_CONFIG_PATH
            ),
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

    let etcd_args = match config.etcd.clone() {
        Some(etcd) => {
            let mut args = vec![];
            if let Some(endpoints) = &etcd.endpoints {
                args.push(format!("--endpoints={}", endpoints.join(",")));
            }
            if let Some(user) = &etcd.user {
                args.push(format!("--user={}", user));
            }
            if let Some(password) = &etcd.password {
                args.push(format!("--password={}", password));
            }
            if let Some(cacert) = &etcd.cacert {
                args.push(format!("--cacert={}", cacert));
            }
            if let Some(cert) = &etcd.cert {
                args.push(format!("--cert={}", cert));
            }
            args
        }
        None => vec![],
    };

    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        match runtime.block_on(async {
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

            println!(
                "[+] Assigning DNS entry: {}.firecracker -> {}",
                name, ip_addr
            );

            let etcd_key = format!("/skydns/firecracker/{}", name);
            let etcd_value = format!("{{\"host\":\"{}\"}}", ip_addr);
            let mut args = vec!["put", &etcd_key, &etcd_value];
            args.extend(etcd_args.iter().map(String::as_str));

            run_command("etcdctl", &args, false)?;

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

pub fn etcd_is_installed() -> Result<bool, Error> {
    let output = run_command("ls", &["/usr/bin/etcd"], false)?;
    Ok(output.status.success())
}

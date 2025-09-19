use std::fs;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Error;
use firecracker_prepare::command::run_command_with_stdout_inherit;

use crate::types::VmOptions;

pub fn setup_tailscale(name: &str, config: &VmOptions) -> Result<(), Error> {
    if let Some(tailscale) = &config.tailscale {
        if let Some(auth_key) = &tailscale.auth_key {
            let len = auth_key.len();
            let display_key = if len > 16 {
                format!("{}****{}", &auth_key[..16], &auth_key[len - 4..])
            } else {
                return Err(anyhow!("Tailscale auth key is too short"));
            };
            println!("[+] Setting up Tailscale with auth key: {}", display_key);
            let key_path =
                get_private_key_path().with_context(|| "Failed to get SSH private key path")?;

            let guest_ip = format!("{}.firecracker", name);
            run_ssh_command(&key_path, &guest_ip, "rm -f /etc/security/namespace.init")?;

            if config.alpine.unwrap_or(false) {
                run_ssh_command(&key_path, &guest_ip, "apk add openrc")?;
            }

            if config.gentoo.unwrap_or(false) {
                run_ssh_command(&key_path, &guest_ip, "emerge --sync")?;
                run_ssh_command(&key_path, &guest_ip, "emerge net-misc/curl")?;
            }

            if config.slackware.unwrap_or(false) {
                // run_ssh_command(&key_path, &guest_ip, "slackpkg update")?;
                run_ssh_command(
                    &key_path,
                    &guest_ip,
                    "yes | slackpkg install nghttp2 brotli zstd libidn2 libpsl cyrus-sasl perl",
                )?;
                run_ssh_command(&key_path, &guest_ip, "update-ca-certificates --fresh")?;
            }

            run_ssh_command(
                &key_path,
                &guest_ip,
                "type tailscaled || curl -fsSL https://tailscale.com/install.sh | sh",
            )?;

            if config.alpine.unwrap_or(false) || config.slackware.unwrap_or(false) {
                run_ssh_command(
                    &key_path,
                    &guest_ip,
                    &format!("tailscale up --auth-key {} --hostname {}", auth_key, name),
                )?;
                run_ssh_command(&key_path, &guest_ip, "rc-status")?;
            } else {
                run_ssh_command(
                    &key_path,
                    &guest_ip,
                    "systemctl enable tailscaled && systemctl start tailscaled || true",
                )?;
                run_ssh_command(
                    &key_path,
                    &guest_ip,
                    &format!("tailscale up --auth-key {} --hostname {}", auth_key, name),
                )?;
                run_ssh_command(&key_path, &guest_ip, "systemctl status tailscaled || true")?;
            }

            run_ssh_command(&key_path, &guest_ip, "tailscale status || true")?;

            println!("[+] Tailscale setup completed.");
            return Ok(());
        }
    }

    println!("[+] Tailscale auth key not provided, skipping Tailscale setup.");
    Ok(())
}

fn run_ssh_command(key_path: &str, guest_ip: &str, command: &str) -> Result<(), Error> {
    run_command_with_stdout_inherit(
        "ssh",
        &[
            "-i",
            key_path,
            "-o",
            "StrictHostKeyChecking=no",
            &format!("root@{}", guest_ip),
            command,
        ],
        false,
    )?;
    Ok(())
}

fn get_private_key_path() -> Result<String, Error> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Failed to get home directory"))?;
    let app_dir = format!("{}/.fireup", home_dir.display());
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
    Ok(key_name)
}

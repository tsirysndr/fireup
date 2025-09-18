use anyhow::Result;

use crate::command::{run_command, run_command_with_stdout_inherit};

pub fn generate_and_copy_ssh_key(key_name: &str, squashfs_root_dir: &str) -> Result<()> {
    let app_dir = crate::config::get_config_dir()?;

    if std::path::Path::new(&format!("{}/{}", app_dir, key_name)).exists() {
        println!(
            "[!] Warning: {} already exists, skipping key generation.",
            key_name
        );
        let pub_key_path = format!("{}/{}.pub", app_dir, key_name);
        let auth_keys_path = format!("{}/root/.ssh/authorized_keys", squashfs_root_dir);
        run_command(
            "mkdir",
            &["-p", &format!("{}/root/.ssh", squashfs_root_dir)],
            true,
        )?;
        run_command("cp", &[&pub_key_path, &auth_keys_path], true)?;
        return Ok(());
    }

    let key_name = format!("{}/{}", app_dir, key_name);
    run_command_with_stdout_inherit("ssh-keygen", &["-f", &key_name, "-N", ""], false)?;

    let pub_key_path = format!("{}.pub", key_name);
    let auth_keys_path = format!("{}/root/.ssh/authorized_keys", squashfs_root_dir);
    run_command("cp", &[&pub_key_path, &auth_keys_path], true)?;
    Ok(())
}

pub fn generate_and_copy_ssh_key_nixos(key_name: &str, squashfs_root_dir: &str) -> Result<String> {
    let app_dir = crate::config::get_config_dir()?;
    const DEFAULT_SSH: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAR4Gvuv3lTpXIYeZTRO22nVEj64uMmlDAdt5+GG80hm tsiry@tsiry-XPS-9320";

    if std::path::Path::new(&format!("{}/{}", app_dir, key_name)).exists() {
        println!(
            "[!] Warning: {} already exists, skipping key generation.",
            key_name
        );
        let pub_key_path = format!("{}/{}.pub", app_dir, key_name);
        let nixos_configuration = format!("{}/etc/nixos/configuration.nix", squashfs_root_dir);
        let public_key = std::fs::read_to_string(&pub_key_path)
            .map_err(|e| anyhow::anyhow!("Failed to read public key: {}", e))?
            .trim()
            .to_string();
        run_command(
            "sed",
            &[
                "-i",
                &format!("s|{}|{}|", DEFAULT_SSH, public_key),
                &nixos_configuration,
            ],
            true,
        )?;

        run_command(
            "mkdir",
            &["-p", &format!("{}/root/.ssh", squashfs_root_dir)],
            true,
        )?;
        run_command(
            "cp",
            &[
                &pub_key_path,
                &format!("{}/root/.ssh/authorized_keys", squashfs_root_dir),
            ],
            true,
        )?;

        return Ok(public_key);
    }

    let key_name = format!("{}/{}", app_dir, key_name);
    run_command_with_stdout_inherit("ssh-keygen", &["-f", &key_name, "-N", ""], false)?;

    let pub_key_path = format!("{}.pub", key_name);
    let nixos_configuration = format!("{}/etc/nixos/configuration.nix", squashfs_root_dir);
    let public_key = std::fs::read_to_string(&pub_key_path)
        .map_err(|e| anyhow::anyhow!("Failed to read public key: {}", e))?
        .trim()
        .to_string();
    run_command(
        "sed",
        &[
            "-i",
            &format!("s|{}|{}|", DEFAULT_SSH, public_key),
            &nixos_configuration,
        ],
        true,
    )?;

    run_command(
        "mkdir",
        &["-p", &format!("{}/root/.ssh", squashfs_root_dir)],
        true,
    )?;
    run_command(
        "cp",
        &[
            &pub_key_path,
            &format!("{}/root/.ssh/authorized_keys", squashfs_root_dir),
        ],
        true,
    )?;

    Ok(public_key)
}

pub fn copy_ssh_keys(ssh_keys: &[String], squashfs_root_dir: &str) -> Result<()> {
    run_command(
        "mkdir",
        &["-p", &format!("{}/root/.ssh", squashfs_root_dir)],
        true,
    )?;

    let auth_keys_path = "/tmp/authorized_keys";
    let mut auth_keys_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&auth_keys_path)
        .map_err(|e| anyhow::anyhow!("Failed to open authorized_keys file: {}", e))?;

    for key in ssh_keys {
        use std::io::Write;
        writeln!(auth_keys_file, "{}", key)
            .map_err(|e| anyhow::anyhow!("Failed to write to authorized_keys file: {}", e))?;
    }

    run_command(
        "cp",
        &[
            &auth_keys_path,
            &format!("{}/root/.ssh/authorized_keys", squashfs_root_dir),
        ],
        true,
    )?;
    std::fs::remove_file(&auth_keys_path)
        .map_err(|e| anyhow::anyhow!("Failed to remove temporary authorized_keys file: {}", e))?;

    Ok(())
}

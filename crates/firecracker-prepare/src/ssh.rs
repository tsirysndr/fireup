use anyhow::Result;

use crate::command::{run_command, run_command_with_stdout_inherit};

pub fn generate_and_copy_ssh_key(key_name: &str, squashfs_root_dir: &str) -> Result<()> {
    if std::path::Path::new(key_name).exists() {
        println!(
            "[!] Warning: {} already exists, skipping key generation.",
            key_name
        );
        let pub_key_path = format!("{}.pub", key_name);
        let auth_keys_path = format!("{}/root/.ssh/authorized_keys", squashfs_root_dir);
        run_command("cp", &[&pub_key_path, &auth_keys_path], true)?;
        return Ok(());
    }

    run_command_with_stdout_inherit("ssh-keygen", &["-f", key_name, "-N", ""], false)?;

    let pub_key_path = format!("{}.pub", key_name);
    let auth_keys_path = format!("{}/root/.ssh/authorized_keys", squashfs_root_dir);
    run_command("cp", &[&pub_key_path, &auth_keys_path], true)?;
    Ok(())
}

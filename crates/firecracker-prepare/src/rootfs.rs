use anyhow::Result;

use crate::command::run_command;

pub fn extract_squashfs(squashfs_file: &str, output_dir: &str) -> Result<()> {
    if std::path::Path::new(output_dir).exists() {
        println!(
            "[!] Warning: {} already exists, skipping extraction.",
            output_dir
        );
        return Ok(());
    }

    println!("Extracting rootfs...");
    run_command("unsquashfs", &["-d", output_dir, squashfs_file], false)?;
    Ok(())
}

pub fn create_ext4_filesystem(squashfs_dir: &str, output_file: &str, size: usize) -> Result<()> {
    run_command("chown", &["-R", "root:root", squashfs_dir], true)?;
    run_command(
        "truncate",
        &["-s", &format!("{}M", size), output_file],
        false,
    )?;
    run_command("mkfs.ext4", &["-d", squashfs_dir, "-F", output_file], true)?;
    Ok(())
}

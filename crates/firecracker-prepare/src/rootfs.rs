use anyhow::Result;

use crate::command::run_command;

pub fn extract_squashfs(squashfs_file: &str, output_dir: &str) -> Result<()> {
    run_command("rm", &["-rf", output_dir], true)?;
    println!("Extracting rootfs...");
    run_command("unsquashfs", &[squashfs_file], false)?;
    Ok(())
}

pub fn create_ext4_filesystem(squashfs_dir: &str, output_file: &str) -> Result<()> {
    run_command("chown", &["-R", "root:root", squashfs_dir], true)?;
    run_command("truncate", &["-s", "400M", output_file], false)?;
    run_command("mkfs.ext4", &["-d", squashfs_dir, "-F", output_file], true)?;
    Ok(())
}

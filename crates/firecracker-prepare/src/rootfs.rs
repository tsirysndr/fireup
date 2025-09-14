use anyhow::Result;

use crate::command::{run_command, run_command_with_stdout_inherit};

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

pub fn create_squashfs(squashfs_dir: &str, output_file: &str) -> Result<()> {
    if std::path::Path::new(output_file).exists() {
        println!(
            "[!] Warning: {} already exists, skipping. Delete it and try again if you want to recreate it.",
            output_file
        );
        return Ok(());
    }
    run_command_with_stdout_inherit("mksquashfs", &[squashfs_dir, output_file], true)?;
    Ok(())
}

pub fn create_overlay_dirs(rootfs_dir: &str) -> Result<()> {
    run_command(
        "mkdir",
        &[
            "-p",
            &format!("{}/overlay/work", rootfs_dir),
            &format!("{}/overlay/root", rootfs_dir),
            &format!("{}/rom", rootfs_dir),
        ],
        true,
    )?;
    Ok(())
}

pub fn add_overlay_init(rootfs_dir: &str) -> Result<()> {
    const OVERLAY_INIT: &str = include_str!("./scripts/overlay-init.sh");
    // add overlay-init script to rootfs/sbin/overlay-init
    println!("Adding overlay-init script...");
    std::fs::write("/tmp/overlay-init", OVERLAY_INIT)?;
    run_command("mkdir", &["-p", &format!("{}/sbin", rootfs_dir)], true)?;
    run_command(
        "mv",
        &["/tmp/overlay-init", &format!("{}/sbin", rootfs_dir)],
        true,
    )?;
    println!("Making overlay-init executable...");
    run_command(
        "chmod",
        &["+x", &format!("{}/sbin/overlay-init", rootfs_dir)],
        true,
    )?;
    Ok(())
}

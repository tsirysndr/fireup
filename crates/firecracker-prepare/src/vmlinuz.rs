use anyhow::Error;
use owo_colors::OwoColorize;
use std::process::Command;

use crate::command::run_command;

pub fn extract_vmlinuz(vmlinuz_file: &str) -> Result<(), Error> {
    if !is_compressed(vmlinuz_file)? {
        return Ok(());
    }

    println!("[*] Extracting vmlinux from compressed vmlinuz...");

    const EXTRACT_VMLINUX: &str = include_str!("./scripts/extract-vmlinux.sh");

    let home_dir = dirs::home_dir().unwrap();
    let bin_dir = home_dir.join(".fireup").join("bin");
    std::fs::create_dir_all(&bin_dir)?;
    let extract_vmlinux_path = bin_dir.join("extract-vmlinux");
    std::fs::write(&extract_vmlinux_path, EXTRACT_VMLINUX)?;

    run_command(
        "chmod",
        &["+x", extract_vmlinux_path.to_str().unwrap()],
        false,
    )?;

    let output = std::path::Path::new(vmlinuz_file)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

    run_command(
        "bash",
        &[
            "-c",
            &format!(
                "{} {} > /tmp/{}",
                extract_vmlinux_path.to_str().unwrap(),
                vmlinuz_file,
                output
            ),
        ],
        false,
    )?;

    std::fs::copy(format!("/tmp/{}", output), vmlinuz_file)?;
    std::fs::remove_file(format!("/tmp/{}", output))?;

    println!("[*] vmlinux extracted to {}", vmlinuz_file.cyan());

    Ok(())
}

fn is_compressed(vmlinuz_file: &str) -> Result<bool, Error> {
    // run command file on vmlinuz_file and check if output contains "bzImage"
    let output = Command::new("file").arg(vmlinuz_file).output()?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    if output_str.contains("bzImage") {
        return Ok(true);
    }
    Ok(false)
}

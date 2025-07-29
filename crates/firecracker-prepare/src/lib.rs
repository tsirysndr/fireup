use anyhow::Result;
use owo_colors::OwoColorize;

pub mod command;
pub mod config;
pub mod downloader;
pub mod rootfs;
pub mod ssh;

pub fn prepare() -> Result<()> {
    let arch = command::run_command("uname", &["-m"], false)?.stdout;
    let arch = String::from_utf8_lossy(&arch).trim().to_string();

    println!("[+] Detected architecture: {}", arch.bright_green());

    let (kernel_file, ubuntu_file, ubuntu_version) = downloader::download_files(&arch)?;

    let app_dir = config::get_config_dir()?;
    let squashfs_root_dir = format!("{}/squashfs_root", app_dir);

    rootfs::extract_squashfs(&ubuntu_file, &squashfs_root_dir)?;

    let ssh_key_name = format!("ubuntu-{}.id_rsa", ubuntu_version);
    ssh::generate_and_copy_ssh_key(&ssh_key_name, &squashfs_root_dir)?;

    let ext4_file = format!("{}/ubuntu-{}.ext4", app_dir, ubuntu_version);

    if !std::path::Path::new(&ext4_file).exists() {
        rootfs::create_ext4_filesystem(&squashfs_root_dir, &ext4_file)?;
    } else {
        println!(
            "[!] {} already exists, skipping ext4 creation.",
            ext4_file.bright_yellow()
        );
    }

    let ssh_key_file = format!("{}/{}", app_dir, ssh_key_name);

    println!("[✓] Kernel: {}", kernel_file.bright_green());
    println!("[✓] Rootfs: {}", ext4_file.bright_green());
    println!("[✓] SSH Key: {}", ssh_key_file.bright_green());

    Ok(())
}

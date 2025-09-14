use anyhow::{anyhow, Context, Result};
use owo_colors::OwoColorize;
use regex::Regex;
use std::path::Path;

use crate::command::{run_command, run_command_with_stdout_inherit};

pub fn download_files(arch: &str) -> Result<(String, String, String)> {
    let app_dir =
        crate::config::get_config_dir().with_context(|| "Failed to get configuration directory")?;
    let kernel_file = download_kernel(arch)?;

    let ci_version = get_ci_version().with_context(|| "Failed to get CI version")?;

    let ubuntu_prefix = format!("firecracker-ci/{}/{}/ubuntu-", ci_version, arch);
    let latest_ubuntu_key =
        get_latest_key("http://spec.ccfc.min.s3.amazonaws.com/", &ubuntu_prefix)?;
    let ubuntu_version = Path::new(&latest_ubuntu_key)
        .file_name()
        .ok_or_else(|| anyhow!("Failed to get ubuntu filename"))?
        .to_string_lossy()
        .to_string();
    let re_version = Regex::new(r"^\d+\.\d+$").with_context(|| "Failed to create version regex")?;
    let ubuntu_version = re_version
        .find(&ubuntu_version)
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| {
            Path::new(&latest_ubuntu_key)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .strip_prefix("ubuntu-")
                .unwrap()
                .strip_suffix(".squashfs")
                .unwrap()
                .to_string()
        });
    let ubuntu_file = format!("{}/ubuntu-{}.squashfs.upstream", app_dir, ubuntu_version);
    download_file(
        &format!(
            "https://s3.amazonaws.com/spec.ccfc.min/{}",
            latest_ubuntu_key
        ),
        &ubuntu_file,
    )?;

    Ok((kernel_file, ubuntu_file, ubuntu_version))
}

fn get_ci_version() -> Result<String> {
    let output = run_command(
        "curl",
        &[
            "-fsSLI",
            "-o",
            "/dev/null",
            "-w",
            "%{url_effective}",
            "https://github.com/firecracker-microvm/firecracker/releases/latest",
        ],
        false,
    )?;
    let url = String::from_utf8_lossy(&output.stdout);
    let version = url
        .split('/')
        .last()
        .ok_or_else(|| anyhow!("Failed to parse version from URL"))?
        .trim();
    let ci_version = version
        .rsplitn(2, '.')
        .last()
        .ok_or_else(|| anyhow!("Failed to parse CI version"))?;
    Ok(ci_version.to_string())
}

pub fn download_kernel(arch: &str) -> Result<String> {
    let app_dir =
        crate::config::get_config_dir().with_context(|| "Failed to get configuration directory")?;
    let ci_version =
        get_ci_version().with_context(|| "Failed to get CI version for kernel download")?;

    let kernel_prefix = format!("firecracker-ci/{}/{}/vmlinux-", ci_version, arch);
    let latest_kernel_key =
        get_latest_key("http://spec.ccfc.min.s3.amazonaws.com/", &kernel_prefix)?;
    let kernel_file = format!(
        "{}/{}",
        app_dir,
        latest_kernel_key.split('/').last().unwrap()
    );

    download_file(
        &format!(
            "https://s3.amazonaws.com/spec.ccfc.min/{}",
            latest_kernel_key
        ),
        &kernel_file,
    )?;

    Ok(kernel_file)
}

fn get_latest_key(url: &str, prefix: &str) -> Result<String> {
    let output = run_command(
        "curl",
        &["-s", &format!("{}?prefix={}&list-type=2", url, prefix)],
        false,
    )?;
    let xml_str = String::from_utf8_lossy(&output.stdout);
    // Match vmlinux-X.Y.Z or ubuntu-X.Y.squashfs
    let re = Regex::new(r"<Key>(firecracker-ci/[^<]+/[^<]+/(?:vmlinux-\d+\.\d+\.\d{1,3}|ubuntu-\d+\.\d+\.squashfs))</Key>")
        .with_context(|| "Failed to create regex")?;

    let mut keys: Vec<String> = re
        .captures_iter(&xml_str)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect();

    if keys.is_empty() {
        return Err(anyhow!("No matching keys found for prefix: {}", prefix));
    }

    // Sort by version number using version sorting similar to sort -V
    keys.sort_by(|a, b| {
        let a_version = Path::new(a)
            .file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.split('-').last())
            .and_then(|v| v.strip_suffix(".squashfs"))
            .unwrap_or("")
            .split('.')
            .map(|n| n.parse::<u32>().unwrap_or(0))
            .collect::<Vec<u32>>();
        let b_version = Path::new(b)
            .file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.split('-').last())
            .and_then(|v| v.strip_suffix(".squashfs"))
            .unwrap_or("")
            .split('.')
            .map(|n| n.parse::<u32>().unwrap_or(0))
            .collect::<Vec<u32>>();

        a_version.cmp(&b_version)
    });

    keys.last()
        .ok_or_else(|| anyhow!("No matching keys found after sorting"))
        .cloned()
}

fn download_file(url: &str, output: &str) -> Result<()> {
    if Path::new(output).exists() {
        println!(
            "File already exists: {}, skipping download.",
            output.bright_green()
        );
        return Ok(());
    }
    println!("Downloading: {}", output.bright_green());
    run_command_with_stdout_inherit("wget", &["-O", output, url], false)?;
    Ok(())
}

pub fn download_alpine_rootfs(minirootfs: &str, arch: &str) -> Result<()> {
    let app_dir = crate::config::get_config_dir()?;
    let output = format!("{}/alpine-{}.tar.gz", app_dir, arch);
    const ALPINE_VERSION: &str = "3.22";
    download_file(&format!("https://mirrors.aliyun.com/alpine/v{}/releases/x86_64/alpine-minirootfs-{}.0-{}.tar.gz", ALPINE_VERSION, ALPINE_VERSION, arch), &output)?;
    run_command("mkdir", &["-p", minirootfs], true)?;
    run_command("tar", &["-xzf", &output, "-C", minirootfs], true)?;
    Ok(())
}

pub fn download_nixos_rootfs(_arch: &str) -> Result<()> {
    let app_dir = crate::config::get_config_dir()?;
    let output = format!("{}/nixos-rootfs.squashfs", app_dir);
    download_file("https://public.rocksky.app/nixos-rootfs.img", &output)?;
    Ok(())
}

pub fn download_fedora_rootfs(_arch: &str) -> Result<()> {
    let app_dir = crate::config::get_config_dir()?;
    let output = format!("{}/fedora-rootfs.squashfs", app_dir);
    download_file("https://public.rocksky.app/fedora-rootfs.img", &output)?;
    Ok(())
}

pub fn download_gentoo_rootfs(_arch: &str) -> Result<()> {
    let app_dir = crate::config::get_config_dir()?;
    let output = format!("{}/gentoo-rootfs.squashfs", app_dir);
    download_file("https://public.rocksky.app/gentoo-rootfs.img", &output)?;
    Ok(())
}

pub fn download_slackware_rootfs(_arch: &str) -> Result<()> {
    let app_dir = crate::config::get_config_dir()?;
    let output = format!("{}/slackware-rootfs.squashfs", app_dir);
    download_file("https://public.rocksky.app/slackware-rootfs.img", &output)?;
    Ok(())
}

pub fn download_opensuse_rootfs(_arch: &str) -> Result<()> {
    let app_dir = crate::config::get_config_dir()?;
    let output = format!("{}/opensuse-rootfs.squashfs", app_dir);
    download_file("https://public.rocksky.app/opensuse-rootfs.img", &output)?;
    Ok(())
}

pub fn download_opensuse_tumbleweed_rootfs(_arch: &str) -> Result<()> {
    let app_dir = crate::config::get_config_dir()?;
    let output = format!("{}/opensuse-tumbleweed-rootfs.squashfs", app_dir);
    download_file(
        "https://public.rocksky.app/opensuse-tumbleweed-rootfs.img",
        &output,
    )?;
    Ok(())
}

pub fn download_almalinux_rootfs(_arch: &str) -> Result<()> {
    let app_dir = crate::config::get_config_dir()?;
    let output = format!("{}/almalinux-rootfs.squashfs", app_dir);
    download_file("https://public.rocksky.app/almalinux-rootfs.img", &output)?;
    Ok(())
}

pub fn download_rockylinux_rootfs(_arch: &str) -> Result<()> {
    let app_dir = crate::config::get_config_dir()?;
    let output = format!("{}/rockylinux-rootfs.squashfs", app_dir);
    download_file("https://public.rocksky.app/rockylinux-rootfs.img", &output)?;
    Ok(())
}

pub fn download_archlinux_rootfs(_arch: &str) -> Result<()> {
    let app_dir = crate::config::get_config_dir()?;
    let output = format!("{}/archlinux-rootfs.squashfs", app_dir);
    download_file("https://public.rocksky.app/archlinux-rootfs.img", &output)?;
    Ok(())
}

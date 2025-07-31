use std::thread;

use anyhow::Error;
use firecracker_prepare::Distro;
use owo_colors::OwoColorize;

use crate::command::run_command;

#[derive(Default, Clone)]
pub struct UpOptions {
    pub debian: Option<bool>,
    pub alpine: Option<bool>,
    pub ubuntu: Option<bool>,
    pub nixos: Option<bool>,
    pub vcpu: u16,
    pub memory: u16,
}

impl Into<Distro> for UpOptions {
    fn into(self) -> Distro {
        if self.debian.unwrap_or(false) {
            Distro::Debian
        } else if self.alpine.unwrap_or(false) {
            Distro::Alpine
        } else if self.nixos.unwrap_or(false) {
            Distro::NixOS
        } else if self.ubuntu.unwrap_or(true) {
            Distro::Ubuntu
        } else {
            panic!("No valid distribution option provided.");
        }
    }
}

pub fn up(options: UpOptions) -> Result<(), Error> {
    check_kvm_support()?;

    firecracker_process::start()?;

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
        if firecracker_process::is_running() {
            println!("[+] Firecracker is running.");
            break;
        }
    }

    firecracker_prepare::prepare(options.clone().into())?;
    firecracker_vm::setup(options.clone().into(), options.vcpu, options.memory)?;
    Ok(())
}

pub fn check_kvm_support() -> Result<(), Error> {
    print!("[+] Checking for kvm support... ");

    if !run_command("sh", &["-c", "lsmod | grep kvm"], false)
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        return Err(anyhow::anyhow!(
            "KVM is not available. Please ensure KVM is enabled in your system."
        ));
    }

    println!("{}", "[✓] OK".bright_green());

    Ok(())
}

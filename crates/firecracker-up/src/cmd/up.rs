use std::thread;

use anyhow::Error;
use firecracker_prepare::PrepareOptions;
use owo_colors::OwoColorize;

use crate::command::run_command;

#[derive(Default, Clone)]
pub struct UpOptions {
    pub debian: Option<bool>,
    pub alpine: Option<bool>,
    pub ubuntu: Option<bool>,
}

impl Into<PrepareOptions> for UpOptions {
    fn into(self) -> PrepareOptions {
        PrepareOptions {
            debian: self.debian,
            alpine: self.alpine,
            ubuntu: self.ubuntu,
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
    firecracker_vm::setup(options.into())?;
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

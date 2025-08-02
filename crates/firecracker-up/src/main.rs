use anyhow::Result;
use clap::{arg, Arg, Command};
use firecracker_vm::types::VmOptions;
use owo_colors::OwoColorize;

use crate::cmd::{down::down, logs::logs, reset::reset, ssh::ssh, status::status, up::up};

pub mod cmd;
pub mod command;
pub mod config;

fn cli() -> Command {
    let banner = format!(
        "{}",
        r#"
     _______           __  __
    / ____(_)_______  / / / /___
   / /_  / / ___/ _ \/ / / / __ \
  / __/ / / /  /  __/ /_/ / /_/ /
 /_/   /_/_/   \___/\____/ .___/
                        /_/
"#
        .yellow()
    );

    Command::new("fireup")
        .version(env!("CARGO_PKG_VERSION"))
        .about(&banner)
        .subcommand(
            Command::new("up")
                .arg(arg!(--debian "Prepare Debian MicroVM").default_value("false"))
                .arg(arg!(--alpine "Prepare Alpine MicroVM").default_value("false"))
                .arg(arg!(--nixos "Prepare NixOS MicroVM").default_value("false"))
                .arg(arg!(--ubuntu "Prepare Ubuntu MicroVM").default_value("true"))
                .arg(arg!(--vcpu <n> "Number of vCPUs"))
                .arg(arg!(--memory <m> "Memory size in MiB"))
                .arg(arg!(--vmlinux <path> "Path to the kernel image"))
                .arg(arg!(--rootfs <path> "Path to the root filesystem image"))
                .arg(
                    Arg::new("boot-args")
                        .long("boot-args")
                        .value_name("ARGS")
                        .help("Override boot arguments"),
                )
                .about("Start Firecracker MicroVM"),
        )
        .subcommand(Command::new("down").about("Stop Firecracker MicroVM"))
        .subcommand(Command::new("status").about("Check the status of Firecracker MicroVM"))
        .subcommand(
            Command::new("logs")
                .arg(
                    arg!(--follow -f "Follow the logs")
                        .short('f')
                        .long("follow")
                        .default_value("false"),
                )
                .about("View the logs of the Firecracker MicroVM"),
        )
        .subcommand(Command::new("ssh").about("SSH into the Firecracker MicroVM"))
        .subcommand(Command::new("reset").about("Reset the Firecracker MicroVM"))
        .arg(arg!(--debian "Prepare Debian MicroVM").default_value("false"))
        .arg(arg!(--alpine "Prepare Alpine MicroVM").default_value("false"))
        .arg(arg!(--nixos "Prepare NixOS MicroVM").default_value("false"))
        .arg(arg!(--ubuntu "Prepare Ubuntu MicroVM").default_value("true"))
        .arg(arg!(--vcpu <n> "Number of vCPUs"))
        .arg(arg!(--memory <m> "Memory size in MiB"))
        .arg(arg!(--vmlinux <path> "Path to the kernel image"))
        .arg(arg!(--rootfs <path> "Path to the root filesystem image"))
        .arg(
            Arg::new("boot-args")
                .long("boot-args")
                .value_name("ARGS")
                .help("Override boot arguments"),
        )
}

fn main() -> Result<()> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("up", args)) => {
            let vcpu = matches
                .get_one::<String>("vcpu")
                .map(|s| s.parse::<u16>().unwrap())
                .unwrap_or(num_cpus::get() as u16);
            let memory = matches
                .get_one::<String>("memory")
                .map(|s| s.parse::<u16>().unwrap())
                .unwrap_or(512);
            let vmlinux = matches.get_one::<String>("vmlinux").cloned();
            let rootfs = matches.get_one::<String>("rootfs").cloned();
            let bootargs = matches.get_one::<String>("boot-args").cloned();
            let options = VmOptions {
                debian: args.get_one::<bool>("debian").copied(),
                alpine: args.get_one::<bool>("alpine").copied(),
                ubuntu: args.get_one::<bool>("ubuntu").copied(),
                nixos: args.get_one::<bool>("nixos").copied(),
                vcpu,
                memory,
                vmlinux,
                rootfs,
                bootargs,
            };
            up(options)?
        }
        Some(("down", _)) => down()?,
        Some(("status", _)) => status()?,
        Some(("logs", args)) => {
            let follow = args.get_one::<bool>("follow").copied().unwrap_or(false);
            logs(follow)?;
        }
        Some(("ssh", _)) => ssh()?,
        Some(("reset", _)) => reset()?,
        _ => {
            let debian = matches.get_one::<bool>("debian").copied().unwrap_or(false);
            let alpine = matches.get_one::<bool>("alpine").copied().unwrap_or(false);
            let nixos = matches.get_one::<bool>("nixos").copied().unwrap_or(false);
            let ubuntu = matches.get_one::<bool>("ubuntu").copied().unwrap_or(false);
            let vcpu = matches
                .get_one::<String>("vcpu")
                .map(|s| s.parse::<u16>().unwrap())
                .unwrap_or(num_cpus::get() as u16);
            let memory = matches
                .get_one::<String>("memory")
                .map(|s| s.parse::<u16>().unwrap())
                .unwrap_or(if nixos { 2048 } else { 512 });
            let vmlinux = matches.get_one::<String>("vmlinux").cloned();
            let rootfs = matches.get_one::<String>("rootfs").cloned();
            let bootargs = matches.get_one::<String>("boot-args").cloned();

            let options = VmOptions {
                debian: Some(debian),
                alpine: Some(alpine),
                ubuntu: Some(ubuntu),
                nixos: Some(nixos),
                vcpu,
                memory,
                vmlinux,
                rootfs,
                bootargs,
            };
            up(options)?
        }
    }

    Ok(())
}

use anyhow::Result;
use clap::{arg, Command};
use owo_colors::OwoColorize;

use crate::cmd::{
    down::down,
    logs::logs,
    reset::reset,
    ssh::ssh,
    status::status,
    up::{up, UpOptions},
};

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
                .arg(arg!(--debian "Prepare Debian rootfs").default_value("false"))
                .arg(arg!(--alpine "Prepare Alpine rootfs").default_value("false"))
                .arg(arg!(--nixos "Prepare NixOS rootfs").default_value("false"))
                .arg(arg!(--ubuntu "Prepare Ubuntu rootfs").default_value("true"))
                .arg(arg!(--vcpu <n> "Number of vCPUs"))
                .arg(arg!(--memory <m> "Memory size in MiB"))
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
        .arg(arg!(--debian "Prepare Debian rootfs").default_value("false"))
        .arg(arg!(--alpine "Prepare Alpine rootfs").default_value("false"))
        .arg(arg!(--nixos "Prepare NixOS rootfs").default_value("false"))
        .arg(arg!(--ubuntu "Prepare Ubuntu rootfs").default_value("true"))
        .arg(arg!(--vcpu <n> "Number of vCPUs"))
        .arg(arg!(--memory <m> "Memory size in MiB"))
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
            let options = UpOptions {
                debian: args.get_one::<bool>("debian").copied(),
                alpine: args.get_one::<bool>("alpine").copied(),
                ubuntu: args.get_one::<bool>("ubuntu").copied(),
                nixos: args.get_one::<bool>("nixos").copied(),
                vcpu,
                memory,
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

            let options = UpOptions {
                debian: Some(debian),
                alpine: Some(alpine),
                ubuntu: Some(ubuntu),
                nixos: Some(nixos),
                vcpu,
                memory,
            };
            up(options)?
        }
    }

    Ok(())
}

use anyhow::Result;
use clap::{arg, Command};
use owo_colors::OwoColorize;

use crate::cmd::{down::down, logs::logs, ssh::ssh, status::status, up::up};

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
        .subcommand(Command::new("up").about("Start Firecracker MicroVM"))
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
}

fn main() -> Result<()> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("up", _)) => up()?,
        Some(("down", _)) => down()?,
        Some(("status", _)) => status()?,
        Some(("logs", args)) => {
            let follow = args.get_one::<bool>("follow").copied().unwrap_or(false);
            logs(follow)?;
        }
        Some(("ssh", _)) => ssh()?,
        _ => up()?,
    }

    Ok(())
}

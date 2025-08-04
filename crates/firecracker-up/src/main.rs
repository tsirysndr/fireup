use anyhow::Result;
use clap::{arg, Arg, Command};
use firecracker_vm::{
    constants::{BRIDGE_DEV, TAP_DEV},
    mac::generate_unique_mac,
    types::VmOptions,
};
use owo_colors::OwoColorize;

use crate::cmd::{
    down::down, init::init, logs::logs, reset::reset, ssh::ssh, status::status, up::up,
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
        .subcommand(Command::new("init").about(
            "Create a new Firecracker MicroVM configuration `fire.toml` in the current directory",
        ))
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
                .arg(arg!(--bridge <name> "Name of the bridge interface").default_value(BRIDGE_DEV))
                .arg(arg!(--tap <name> "Name of the tap interface").default_value(TAP_DEV))
                .arg(
                    Arg::new("mac-address")
                        .long("mac-address")
                        .value_name("MAC")
                        .help("MAC address for the network interface"),
                )
                .arg(
                    Arg::new("api-socket")
                        .long("api-socket")
                        .value_name("path")
                        .help("Path to the Firecracker API socket"),
                )
                .arg(
                    Arg::new("boot-args")
                        .long("boot-args")
                        .value_name("ARGS")
                        .help("Override boot arguments"),
                )
                .about("Start Firecracker MicroVM"),
        )
        .subcommand(
            Command::new("down")
                .arg(arg!([name] "Name of the Firecracker MicroVM to reset").required(false))
                .about("Stop Firecracker MicroVM"),
        )
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
        .subcommand(
            Command::new("ssh")
                .arg(arg!([name] "Name of the Firecracker MicroVM to SSH into"))
                .about("SSH into the Firecracker MicroVM"),
        )
        .subcommand(
            Command::new("reset")
                .arg(arg!([name] "Name of the Firecracker MicroVM to reset").required(false))
                .about("Reset the Firecracker MicroVM"),
        )
        .arg(arg!(--debian "Prepare Debian MicroVM").default_value("false"))
        .arg(arg!(--alpine "Prepare Alpine MicroVM").default_value("false"))
        .arg(arg!(--nixos "Prepare NixOS MicroVM").default_value("false"))
        .arg(arg!(--ubuntu "Prepare Ubuntu MicroVM").default_value("true"))
        .arg(arg!(--vcpu <n> "Number of vCPUs"))
        .arg(arg!(--memory <m> "Memory size in MiB"))
        .arg(arg!(--vmlinux <path> "Path to the kernel image"))
        .arg(arg!(--rootfs <path> "Path to the root filesystem image"))
        .arg(arg!(--bridge <name> "Name of the bridge interface").default_value(BRIDGE_DEV))
        .arg(arg!(--tap <name> "Name of the tap interface").default_value(TAP_DEV))
        .arg(
            Arg::new("mac-address")
                .long("mac-address")
                .value_name("MAC")
                .help("MAC address for the network interface"),
        )
        .arg(
            Arg::new("api-socket")
                .long("api-socket")
                .value_name("path")
                .help("Path to the Firecracker API socket"),
        )
        .arg(
            Arg::new("boot-args")
                .long("boot-args")
                .value_name("ARGS")
                .help("Override boot arguments"),
        )
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = cli().get_matches();

    let pool = firecracker_state::create_connection_pool().await?;
    let vm_name = names::Generator::default().next().unwrap();
    let default_socket = format!("/tmp/firecracker-{}.sock", vm_name);
    let default_mac = generate_unique_mac();

    match matches.subcommand() {
        Some(("init", _)) => init()?,
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
            let bridge = args.get_one::<String>("bridge").cloned().unwrap();
            let tap = args.get_one::<String>("tap").cloned().unwrap();
            let api_socket = args
                .get_one::<String>("api-socket")
                .cloned()
                .unwrap_or(default_socket);
            let mac_address = args
                .get_one::<String>("mac-address")
                .cloned()
                .unwrap_or(default_mac);
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
                bridge,
                tap,
                api_socket,
                mac_address,
            };
            up(options).await?
        }
        Some(("down", args)) => {
            let name = args.get_one::<String>("name").cloned().unwrap();
            let api_socket = format!("/tmp/firecracker-{}.sock", name);
            down(&VmOptions {
                api_socket,
                ..Default::default()
            })?
        }
        Some(("status", _)) => status()?,
        Some(("logs", args)) => {
            let follow = args.get_one::<bool>("follow").copied().unwrap_or(false);
            logs(follow)?;
        }
        Some(("ssh", args)) => {
            let name = args.get_one::<String>("name").cloned();
            ssh(pool, name).await?
        }
        Some(("reset", args)) => {
            let name = args.get_one::<String>("name").cloned().unwrap();
            let api_socket = format!("/tmp/firecracker-{}.sock", name);
            reset(VmOptions {
                api_socket,
                ..Default::default()
            })?
        }
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
            let bridge = matches.get_one::<String>("bridge").cloned().unwrap();
            let tap = matches.get_one::<String>("tap").cloned().unwrap();
            let api_socket = matches
                .get_one::<String>("api-socket")
                .cloned()
                .unwrap_or(default_socket);
            let mac_address = matches
                .get_one::<String>("mac-address")
                .cloned()
                .unwrap_or(default_mac);

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
                bridge,
                tap,
                api_socket,
                mac_address,
            };
            up(options).await?
        }
    }

    Ok(())
}

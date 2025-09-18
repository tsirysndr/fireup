use anyhow::Result;
use clap::{arg, Arg, Command};
use firecracker_vm::{constants::BRIDGE_DEV, mac::generate_unique_mac, types::VmOptions};
use owo_colors::OwoColorize;

use crate::cmd::{
    down::down, init::init, logs::logs, ps::list_all_instances, reset::reset, rm::remove,
    serve::serve, ssh::ssh, start::start, status::status, stop::stop, up::up,
};

pub mod cmd;
pub mod command;
pub mod config;
pub mod date;

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
            Command::new("ps")
                .alias("list")
                .arg(arg!(-a --all "Show all Firecracker MicroVM instances").default_value("false"))
                .about("List all Firecracker MicroVM instances"),
        )
        .subcommand(
            Command::new("start")
                .arg(arg!(<name> "Name of the Firecracker MicroVM to start").required(true))
                .about("Start Firecracker MicroVM"),
        )
        .subcommand(
            Command::new("stop")
                .arg(arg!([name] "Name of the Firecracker MicroVM to stop").required(false))
                .about("Stop Firecracker MicroVM"),
        )
        .subcommand(
            Command::new("restart")
                .arg(arg!(<name> "Name of the Firecracker MicroVM to restart").required(true))
                .about("Restart Firecracker MicroVM"),
        )
        .subcommand(
            Command::new("up")
                .arg(arg!(--debian "Prepare Debian MicroVM").default_value("false"))
                .arg(arg!(--alpine "Prepare Alpine MicroVM").default_value("false"))
                .arg(arg!(--nixos "Prepare NixOS MicroVM").default_value("false"))
                .arg(arg!(--fedora "Prepare Fedora MicroVM").default_value("false"))
                .arg(arg!(--gentoo "Prepare Gentoo MicroVM").default_value("false"))
                .arg(arg!(--slackware "Prepare Slackware MicroVM").default_value("false"))
                .arg(arg!(--opensuse "Prepare OpenSUSE MicroVM").default_value("false"))
                .arg(
                    Arg::new("opensuse-tumbleweed")
                        .help("Prepare OpenSUSE Tumbleweed MicroVM")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(arg!(--almalinux "Prepare AlmaLinux MicroVM").default_value("false"))
                .arg(arg!(--rockylinux "Prepare RockyLinux MicroVM").default_value("false"))
                .arg(arg!(--archlinux "Prepare ArchLinux MicroVM").default_value("false"))
                .arg(arg!(--ubuntu "Prepare Ubuntu MicroVM").default_value("true"))
                .arg(arg!(--vcpu <n> "Number of vCPUs"))
                .arg(arg!(--memory <m> "Memory size in MiB"))
                .arg(arg!(--vmlinux <path> "Path to the kernel image"))
                .arg(arg!(--rootfs <path> "Path to the root filesystem image"))
                .arg(arg!(--bridge <name> "Name of the bridge interface").default_value(BRIDGE_DEV))
                .arg(arg!(--tap <name> "Name of the tap interface").default_value(""))
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
                .arg(
                    Arg::new("ssh-keys")
                        .long("ssh-keys")
                        .value_name("SSH_KEYS")
                        .help("Comma-separated list of SSH public keys to add to the VM"),
                )
                .about("Start a new Firecracker MicroVM"),
        )
        .subcommand(Command::new("down").about("Stop Firecracker MicroVM"))
        .subcommand(
            Command::new("status")
                .arg(arg!([name] "Name of the Firecracker MicroVM to check status").required(false))
                .about("Check the status of Firecracker MicroVM"),
        )
        .subcommand(
            Command::new("logs")
                .arg(
                    arg!(-f --follow "Follow the logs")
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
        .subcommand(
            Command::new("rm")
                .arg(arg!(<name> "Name or ID of the Firecracker MicroVM to delete").required(true))
                .about("Delete the Firecracker MicroVM"),
        )
        .subcommand(
            Command::new("serve")
                .about("Start fireup HTTP API server")
                .arg(arg!(--host <host> "Host to bind the server"))
                .arg(arg!(--port <port> "Port to bind the server")),
        )
        .arg(arg!(--debian "Prepare Debian MicroVM").default_value("false"))
        .arg(arg!(--alpine "Prepare Alpine MicroVM").default_value("false"))
        .arg(arg!(--nixos "Prepare NixOS MicroVM").default_value("false"))
        .arg(arg!(--fedora "Prepare Fedora MicroVM").default_value("false"))
        .arg(arg!(--gentoo "Prepare Gentoo MicroVM").default_value("false"))
        .arg(arg!(--slackware "Prepare Slackware MicroVM").default_value("false"))
        .arg(arg!(--opensuse "Prepare OpenSUSE MicroVM").default_value("false"))
        .arg(
            Arg::new("opensuse-tumbleweed")
                .long("opensuse-tumbleweed")
                .help("Prepare OpenSUSE Tumbleweed MicroVM")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(arg!(--almalinux "Prepare AlmaLinux MicroVM").default_value("false"))
        .arg(arg!(--rockylinux "Prepare RockyLinux MicroVM").default_value("false"))
        .arg(arg!(--archlinux "Prepare ArchLinux MicroVM").default_value("false"))
        .arg(arg!(--ubuntu "Prepare Ubuntu MicroVM").default_value("true"))
        .arg(arg!(--vcpu <n> "Number of vCPUs"))
        .arg(arg!(--memory <m> "Memory size in MiB"))
        .arg(arg!(--vmlinux <path> "Path to the kernel image"))
        .arg(arg!(--rootfs <path> "Path to the root filesystem image"))
        .arg(arg!(--bridge <name> "Name of the bridge interface").default_value(BRIDGE_DEV))
        .arg(arg!(--tap <name> "Name of the tap interface").default_value(""))
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
        .arg(
            Arg::new("ssh-keys")
                .long("ssh-keys")
                .value_name("SSH_KEYS")
                .help("Comma-separated list of SSH public keys to add to the VM"),
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
        Some(("ps", args)) => {
            let all = args.get_one::<bool>("all").copied().unwrap_or(false);
            list_all_instances(all).await?;
        }
        Some(("stop", args)) => {
            let name = args.get_one::<String>("name").cloned().unwrap();
            stop(&name).await?;
        }
        Some(("start", args)) => {
            let name = args.get_one::<String>("name").cloned().unwrap();
            start(&name).await?;
        }
        Some(("restart", args)) => {
            let name = args.get_one::<String>("name").cloned().unwrap();
            stop(&name).await?;
            start(&name).await?;
        }
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
            let ssh_keys = args
                .get_one::<String>("ssh-keys")
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect());
            let options = VmOptions {
                debian: args.get_one::<bool>("debian").copied(),
                alpine: args.get_one::<bool>("alpine").copied(),
                ubuntu: args.get_one::<bool>("ubuntu").copied(),
                nixos: args.get_one::<bool>("nixos").copied(),
                fedora: args.get_one::<bool>("fedora").copied(),
                gentoo: args.get_one::<bool>("gentoo").copied(),
                slackware: args.get_one::<bool>("slackware").copied(),
                opensuse: args.get_one::<bool>("opensuse").copied(),
                opensuse_tumbleweed: args.get_one::<bool>("opensuse-tumbleweed").copied(),
                almalinux: args.get_one::<bool>("almalinux").copied(),
                rockylinux: args.get_one::<bool>("rockylinux").copied(),
                archlinux: args.get_one::<bool>("archlinux").copied(),
                vcpu,
                memory,
                vmlinux,
                rootfs,
                bootargs,
                bridge,
                tap,
                api_socket,
                mac_address,
                etcd: None,
                ssh_keys,
            };
            up(options).await?
        }
        Some(("down", _)) => down().await?,
        Some(("status", args)) => {
            let name = args.get_one::<String>("name").cloned();
            status(name).await?;
        }
        Some(("logs", args)) => {
            let follow = args.get_one::<bool>("follow").copied().unwrap_or(false);
            logs(follow)?;
        }
        Some(("ssh", args)) => {
            let name = args.get_one::<String>("name").cloned();
            ssh(pool, name).await?
        }
        Some(("reset", args)) => {
            let name = args.get_one::<String>("name").cloned();
            let api_socket = match name {
                Some(name) => format!("/tmp/firecracker-{}.sock", name),
                None => String::from(""),
            };
            reset(VmOptions {
                api_socket,
                ..Default::default()
            })
            .await?
        }
        Some(("rm", args)) => {
            let name = args.get_one::<String>("name").cloned().unwrap();
            remove(&name).await?
        }
        Some(("serve", _)) => serve().await?,
        _ => {
            let debian = matches.get_one::<bool>("debian").copied().unwrap_or(false);
            let alpine = matches.get_one::<bool>("alpine").copied().unwrap_or(false);
            let nixos = matches.get_one::<bool>("nixos").copied().unwrap_or(false);
            let ubuntu = matches.get_one::<bool>("ubuntu").copied().unwrap_or(false);
            let fedora = matches.get_one::<bool>("fedora").copied().unwrap_or(false);
            let gentoo = matches.get_one::<bool>("gentoo").copied().unwrap_or(false);
            let slackware = matches
                .get_one::<bool>("slackware")
                .copied()
                .unwrap_or(false);
            let opensuse = matches
                .get_one::<bool>("opensuse")
                .copied()
                .unwrap_or(false);
            let opensuse_tumbleweed = matches
                .get_one::<bool>("opensuse-tumbleweed")
                .copied()
                .unwrap_or(false);
            let almalinux = matches
                .get_one::<bool>("almalinux")
                .copied()
                .unwrap_or(false);
            let rockylinux = matches
                .get_one::<bool>("rockylinux")
                .copied()
                .unwrap_or(false);
            let archlinux = matches
                .get_one::<bool>("archlinux")
                .copied()
                .unwrap_or(false);

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
            let ssh_keys = matches
                .get_one::<String>("ssh-keys")
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect());

            let options = VmOptions {
                debian: Some(debian),
                alpine: Some(alpine),
                ubuntu: Some(ubuntu),
                nixos: Some(nixos),
                fedora: Some(fedora),
                gentoo: Some(gentoo),
                slackware: Some(slackware),
                opensuse: Some(opensuse),
                opensuse_tumbleweed: Some(opensuse_tumbleweed),
                almalinux: Some(almalinux),
                rockylinux: Some(rockylinux),
                archlinux: Some(archlinux),
                vcpu,
                memory,
                vmlinux,
                rootfs,
                bootargs,
                bridge,
                tap,
                api_socket,
                mac_address,
                etcd: None,
                ssh_keys,
            };
            up(options).await?
        }
    }

    Ok(())
}

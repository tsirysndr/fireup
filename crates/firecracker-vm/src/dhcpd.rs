use anyhow::Error;

use crate::{command::run_command, constants::BRIDGE_DEV, types::VmOptions};

pub const DHCPD_CONFIG_PATH: &str = "/etc/kea/kea-dhcp4.conf";

pub fn setup_kea_dhcp(_config: &VmOptions) -> Result<(), Error> {
    println!("[+] Checking if isc-kea-dhcp-server is installed...");
    if is_kea_dhcp_installed().is_err() {
        run_command(
            "apt-get",
            &[
                "install",
                "-y",
                "kea-dhcp4-server",
                "kea-admin",
                "kea-common",
                "etcd-client",
                "etcd-server",
            ],
            true,
        )?;
    }

    const KEA_MQTT_HOOK_SH: &str = include_str!("./scripts/kea-mqtt-hook.sh");
    println!("[+] Installing kea-mqtt-hook.sh script...");
    std::fs::write("/tmp/kea-mqtt-hook.sh", KEA_MQTT_HOOK_SH)?;
    run_command("cp", &["/tmp/kea-mqtt-hook.sh", "/usr/local/bin"], true)?;
    run_command("chmod", &["a+x", "/usr/local/bin/kea-mqtt-hook.sh"], true)?;
    run_command("rm", &["/tmp/kea-mqtt-hook.sh"], false)?;

    println!("[+] Setting up AppArmor for kea-dhcp4...");
    std::fs::write(
        "/tmp/usr.sbin.kea-dhcp4",
        include_str!("./apparmor/usr.sbin.kea-dhcp4"),
    )?;
    run_command("cp", &["/tmp/usr.sbin.kea-dhcp4", "/etc/apparmor.d/"], true)?;
    run_command("rm", &["/tmp/usr.sbin.kea-dhcp4"], false)?;
    run_command(
        "apparmor_parser",
        &["-r", "/etc/apparmor.d/usr.sbin.kea-dhcp4"],
        true,
    )?;

    let kea_dhcp_config: &str = &format!(
        r#"
{{
  "Dhcp4": {{
    "valid-lifetime": 4000,
    "renew-timer": 1000,
    "rebind-timer": 2000,
    "interfaces-config": {{
      "interfaces": [ "{}" ]
    }},
    "lease-database": {{
      "type": "memfile",
      "lfc-interval": 3600
    }},
    "subnet4": [
      {{
        "subnet": "172.16.0.0/24",
        "pools": [ {{ "pool": "172.16.0.2 - 172.16.0.150" }} ],
        "option-data": [
          {{ "name": "routers", "data": "172.16.0.1" }},
          {{ "name": "domain-name-servers", "data": "172.16.0.1" }}
        ]
    }}
    ],
    "hooks-libraries": [
      {{
        "library": "/usr/lib/x86_64-linux-gnu/kea/hooks/libdhcp_run_script.so",
        "parameters": {{
          "name": "/usr/local/bin/kea-mqtt-hook.sh"
         }}
      }}
    ],
    "loggers": [
      {{
        "name": "kea-dhcp4",
        "severity": "DEBUG",
        "debuglevel": 99
      }}
    ]
  }}
}}
"#,
        BRIDGE_DEV
    );

    run_command(
        "sh",
        &[
            "-c",
            &format!("echo '{}' > {}", kea_dhcp_config, DHCPD_CONFIG_PATH),
        ],
        true,
    )?;

    restart_kea_dhcp()?;

    Ok(())
}

pub fn restart_kea_dhcp() -> Result<(), Error> {
    println!("[+] Starting kea-dhcp4-server...");

    let dummy_is_up = run_command("ip", &["link", "show", "dummy0"], false)
        .map(|output| output.status.success())
        .unwrap_or(false);
    if !dummy_is_up {
        println!("[+] Creating dummy0 interface...");
        run_command("ip", &["link", "add", "dummy0", "type", "dummy"], true)?;
        run_command("ip", &["link", "set", "dummy0", "up"], true)?;
        run_command("ip", &["link", "set", "dummy0", "master", BRIDGE_DEV], true)?;
    }

    run_command("systemctl", &["enable", "kea-dhcp4-server"], true)?;
    run_command("systemctl", &["daemon-reload"], true)?;
    run_command("systemctl", &["stop", "kea-dhcp4-server"], true)?;
    run_command("systemctl", &["start", "kea-dhcp4-server"], true)?;
    println!("[✓] kea-dhcp4-server started successfully.");
    Ok(())
}

pub fn is_kea_dhcp_installed() -> Result<bool, Error> {
    let output = run_command("which", &["kea-dhcp4"], false)?;
    Ok(output.status.success())
}

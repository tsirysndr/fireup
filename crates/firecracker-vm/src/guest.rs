use crate::{command::run_command, constants::BRIDGE_IP};
use anyhow::Result;

pub fn configure_guest_network(key_path: &str, guest_ip: &str, is_nixos: bool) -> Result<()> {
    println!("[+] Configuring network in guest...");
    const MAX_RETRIES: u32 = 500;
    let mut retries = 0;
    loop {
        if run_command(
            "ssh",
            &[
                "-i",
                key_path,
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                &format!("root@{}", guest_ip),
                &match is_nixos {
                    true => "uname -a".into(),
                    false => format!("echo 'nameserver {}' > /etc/resolv.conf", BRIDGE_IP),
                },
            ],
            false,
        )
        .is_ok()
            || retries >= MAX_RETRIES
        {
            if retries >= MAX_RETRIES {
                println!(
                    "[-] Max retries reached. Failed to configure network in guest. {}",
                    guest_ip
                );
            } else {
                println!("[+] Network configured in guest.");
            }
            break;
        }
        println!("[-] Waiting for ssh to be available...");
        std::thread::sleep(std::time::Duration::from_millis(100));
        retries += 1;
    }
    Ok(())
}

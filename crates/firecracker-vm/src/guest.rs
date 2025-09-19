use crate::{command::run_command, constants::BRIDGE_IP};
use anyhow::Result;

pub fn configure_guest_network(key_path: &str, guest_ip: &str) -> Result<()> {
    println!("[+] Configuring network in guest...");
    const MAX_RETRIES: u32 = 20;
    let mut retries = 0;
    loop {
        if run_command(
            "ssh",
            &[
                "-i",
                key_path,
                "-o",
                "StrictHostKeyChecking=no",
                &format!("root@{}", guest_ip),
                &format!("echo 'nameserver {}' > /etc/resolv.conf", BRIDGE_IP),
            ],
            false,
        )
        .is_ok()
            || retries >= MAX_RETRIES
        {
            break;
        }
        println!("[-] Waiting for ssh to be available...");
        std::thread::sleep(std::time::Duration::from_millis(100));
        retries += 1;
    }
    Ok(())
}

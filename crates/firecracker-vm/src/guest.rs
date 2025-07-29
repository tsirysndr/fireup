use crate::command::run_command;
use crate::constants::GUEST_IP;
use anyhow::Result;

pub fn configure_guest_network(key_name: &str) -> Result<()> {
    println!("[+] Configuring network in guest...");
    if let Err(err) = run_command(
        "ssh",
        &[
            "-i",
            key_name,
            "-o",
            "StrictHostKeyChecking=no",
            &format!("root@{}", GUEST_IP),
            "ip route add default via 172.16.0.1 dev eth0",
        ],
        false,
    ) {
        println!("[-] Failed to set default route: {}", err);
    }
    run_command(
        "ssh",
        &[
            "-i",
            key_name,
            "-o",
            "StrictHostKeyChecking=no",
            &format!("root@{}", GUEST_IP),
            "echo 'nameserver 8.8.8.8' > /etc/resolv.conf",
        ],
        false,
    )?;
    Ok(())
}

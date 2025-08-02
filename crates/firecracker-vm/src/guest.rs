use crate::{
    command::run_command,
    constants::{BRIDGE_IP, GUEST_IP},
};
use anyhow::Result;

pub fn configure_guest_network(key_name: &str) -> Result<()> {
    println!("[+] Configuring network in guest...");
    run_command(
        "ssh",
        &[
            "-i",
            key_name,
            "-o",
            "StrictHostKeyChecking=no",
            &format!("root@{}", GUEST_IP),
            &format!("echo 'nameserver {}' > /etc/resolv.conf", BRIDGE_IP),
        ],
        false,
    )?;
    Ok(())
}

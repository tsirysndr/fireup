use std::process;

use anyhow::Error;
use firecracker_state::{ repo};
use serde_json::json;

pub async fn inspect_microvm(id: &str) -> Result<(), Error> {
    let pool = firecracker_state::create_connection_pool().await?;
    let vm = repo::virtual_machine::find(&pool, id).await?;
    if vm.is_none() {
        println!("[!] No virtual machine found with the name: {}", id);
        process::exit(1);
    }

    let vm = vm.unwrap();
    let vm = json!({
        "id": vm.id,
        "name": vm.name,
        "image": vm.distro,
        "vcpu": vm.vcpu,
        "memory": vm.memory,
        "vmlinux": vm.vmlinux,
        "rootfs": vm.rootfs,
        "bootargs": vm.bootargs,
        "bridge": vm.bridge,
        "tap": vm.tap,
        "api_socket": vm.api_socket,
        "mac_address": vm.mac_address,
        "ssh_keys": vm.ssh_keys,
        "status": vm.status,
        "pid": vm.pid,
        "ip_address": vm.ip_address,
        "project_dir": vm.project_dir,
        "created_at": vm.created_at.to_rfc3339(),
        "updated_at": vm.updated_at.to_rfc3339(),
    });

    let vm_json = serde_json::to_string_pretty(&vm)?;
    println!("{}", vm_json);

    Ok(())
}

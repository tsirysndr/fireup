use std::process;

use anyhow::Error;
use firecracker_state::repo;
use firecracker_vm::types::VmOptions;

use crate::cmd::up::up;

pub async fn start(name: &str) -> Result<(), Error> {
    let pool = firecracker_state::create_connection_pool().await?;
    let vm = repo::virtual_machine::find(&pool, name).await?;
    if vm.is_none() {
        println!("[!] No virtual machine found with the name: {}", name);
        process::exit(1);
    }

    let vm = vm.unwrap();

    up(VmOptions {
        debian: Some(vm.distro == "debian"),
        alpine: Some(vm.distro == "alpine"),
        ubuntu: Some(vm.distro == "ubuntu"),
        nixos: Some(vm.distro == "nixos"),
        vcpu: vm.vcpu,
        memory: vm.memory,
        vmlinux: vm.vmlinux,
        rootfs: vm.rootfs,
        bootargs: vm.bootargs,
        bridge: vm.bridge,
        tap: vm.tap,
        api_socket: vm.api_socket,
        mac_address: vm.mac_address,
    })
    .await?;

    Ok(())
}

use std::process;

use anyhow::Error;
use fire_config::TailscaleOptions;
use firecracker_state::repo;
use firecracker_vm::types::VmOptions;

use crate::cmd::up::up;

pub async fn start(name: &str, tailscale_auth_key: Option<String>) -> Result<(), Error> {
    let etcd = match fire_config::read_config() {
        Ok(config) => config.etcd,
        Err(_) => None,
    };
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
        fedora: Some(vm.distro == "fedora"),
        gentoo: Some(vm.distro == "gentoo"),
        slackware: Some(vm.distro == "slackware"),
        opensuse: Some(vm.distro == "opensuse"),
        opensuse_tumbleweed: Some(vm.distro == "opensuse-tumbleweed"),
        almalinux: Some(vm.distro == "almalinux"),
        rockylinux: Some(vm.distro == "rockylinux"),
        archlinux: Some(vm.distro == "archlinux"),
        vcpu: vm.vcpu,
        memory: vm.memory,
        vmlinux: vm.vmlinux,
        rootfs: vm.rootfs,
        bootargs: vm.bootargs,
        bridge: vm.bridge,
        tap: vm.tap,
        api_socket: vm.api_socket,
        mac_address: vm.mac_address,
        etcd,
        ssh_keys: vm
            .ssh_keys
            .map(|keys| keys.split(',').map(|s| s.to_string()).collect()),
        tailscale: tailscale_auth_key.map(|key| TailscaleOptions {
            auth_key: Some(key),
        }),
    })
    .await?;

    Ok(())
}

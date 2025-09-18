use std::{sync::Arc, thread};

use crate::types::microvm::CreateMicroVM;
use anyhow::Error;
use firecracker_state::{entity::virtual_machine::VirtualMachine, repo};
use firecracker_vm::{constants::BRIDGE_DEV, types::VmOptions};
use owo_colors::OwoColorize;
use sqlx::{Pool, Sqlite};

pub async fn create_microvm(
    pool: Arc<Pool<Sqlite>>,
    params: CreateMicroVM,
) -> Result<VirtualMachine, Error> {
    let mut options: VmOptions = params.into();

    if options.api_socket.is_empty() {
        let vm_name = names::Generator::default().next().unwrap();
        options.api_socket = format!("/tmp/firecracker-{}.sock", vm_name);
    }

    options.bridge = BRIDGE_DEV.into();

    let vm = start(pool, options, None).await?;
    Ok(vm)
}

pub async fn delete_microvm(
    pool: Arc<Pool<Sqlite>>,
    id: &str,
) -> Result<Option<VirtualMachine>, Error> {
    let vm = repo::virtual_machine::find(&pool, id).await?;
    if vm.is_none() {
        println!("[!] No virtual machine found with the name: {}", id);
        return Ok(None);
    }

    let mut vm = vm.unwrap();
    firecracker_process::stop(Some(vm.name.clone())).await?;
    repo::virtual_machine::delete(&pool, id).await?;
    vm.status = "DELETED".into();
    Ok(Some(vm))
}

pub async fn start_microvm(pool: Arc<Pool<Sqlite>>, id: &str) -> Result<VirtualMachine, Error> {
    let vm = repo::virtual_machine::find(&pool, id).await?;
    if vm.is_none() {
        println!("[!] No virtual machine found with the name: {}", id);
        return Err(Error::msg(format!(
            "No virtual machine found with the given id: {}",
            id
        )));
    }

    let vm = vm.unwrap();

    let options = VmOptions {
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
        vmlinux: vm.vmlinux.clone(),
        rootfs: vm.rootfs.clone(),
        bootargs: vm.bootargs.clone(),
        bridge: vm.bridge.clone(),
        tap: vm.tap.clone(),
        api_socket: vm.api_socket.clone(),
        mac_address: vm.mac_address.clone(),
        etcd: None,
        ssh_keys: vm
            .ssh_keys
            .map(|keys| keys.split(',').map(|s| s.to_string()).collect()),
    };

    let vm = start(pool, options, Some(vm.id)).await?;

    Ok(vm)
}

pub async fn stop_microvm(
    pool: Arc<Pool<Sqlite>>,
    id: &str,
) -> Result<Option<VirtualMachine>, Error> {
    let vm = repo::virtual_machine::find(&pool, id).await?;
    if vm.is_none() {
        println!("[!] No virtual machine found with the name: {}", id);
        return Ok(None);
    }

    let mut vm = vm.unwrap();
    firecracker_process::stop(Some(vm.name.clone())).await?;
    repo::virtual_machine::update_status(&pool, id, "STOPPED").await?;
    vm.status = "STOPPED".into();
    Ok(Some(vm))
}

async fn start(
    pool: Arc<Pool<Sqlite>>,
    mut options: VmOptions,
    vm_id: Option<String>,
) -> Result<VirtualMachine, Error> {
    let vms = repo::virtual_machine::all(&pool).await?;
    if options.tap.is_empty() {
        let vms = vms
            .into_iter()
            .filter(|vm| vm.tap.starts_with("tap"))
            .collect::<Vec<_>>();
        options.tap = format!("tap{}", vms.len());

        while vms.iter().any(|vm| vm.tap == options.tap) {
            let tap_num: u32 = options
                .tap
                .trim_start_matches("tap")
                .parse::<u32>()
                .unwrap_or(0)
                .checked_add(1)
                .unwrap_or(0);
            options.tap = format!("tap{}", tap_num);
        }
    } else {
        if vms
            .iter()
            .any(|vm| vm.tap == options.tap && vm.api_socket != options.api_socket)
        {
            println!(
                "[!] Tap device name {} is already in use. Please choose a different name.",
                options.tap.cyan()
            );
            return Err(Error::msg("Tap device name already in use"));
        }
    }

    let pid = firecracker_process::start(&options).await?;

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
        if firecracker_process::is_running() {
            println!("[+] Firecracker is running.");
            break;
        }
    }

    let mut ssh_keys = options.ssh_keys.clone();
    if let Some(vm_id) = vm_id.clone() {
        let vm = repo::virtual_machine::find(&pool, &vm_id).await?;
        if ssh_keys.is_none() {
            ssh_keys = vm.and_then(|vm| {
                vm.ssh_keys
                    .map(|keys| keys.split(',').map(|s| s.to_string()).collect())
            });
        }
    }

    firecracker_prepare::prepare(options.clone().into(), options.vmlinux.clone(), ssh_keys)?;
    let vm_id = firecracker_vm::setup(&options, pid, vm_id).await?;
    let vm = repo::virtual_machine::find(&pool, &vm_id)
        .await?
        .ok_or_else(|| Error::msg("Failed to retrieve the created VM"))?;
    Ok(vm)
}

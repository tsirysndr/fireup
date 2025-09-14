use anyhow::Error;

use crate::date::{format_duration_ago, format_status};

pub async fn list_all_instances(all: bool) -> Result<(), Error> {
    let pool = firecracker_state::create_connection_pool().await?;
    let mut vms = firecracker_state::repo::virtual_machine::all(&pool).await?;
    if !all {
        vms = vms
            .into_iter()
            .filter(|vm| vm.status == "RUNNING")
            .collect::<Vec<_>>();
    }

    if vms.is_empty() {
        println!("No Firecracker MicroVM instances found.");
        return Ok(());
    }
    let distro_length = vms.iter().map(|vm| vm.distro.len()).max().unwrap_or(10) + 2;
    let name_length = vms
        .iter()
        .map(|vm| vm.name.len())
        .max()
        .unwrap_or(10)
        .max(10)
        + 2;
    let vcpu_length = vms
        .iter()
        .map(|vm| vm.vcpu.to_string().len())
        .max()
        .unwrap_or(10)
        + 2;
    let memory_length = vms
        .iter()
        .map(|vm| format!("{} MiB", vm.memory).len())
        .max()
        .unwrap_or(10)
        + 2;
    let status_length = vms
        .iter()
        .map(|vm| format_status(&vm.status, vm.updated_at).len())
        .max()
        .unwrap_or(10);
    let pid_length = vms
        .iter()
        .map(|vm| vm.pid.unwrap_or(0).to_string().len())
        .max()
        .unwrap_or(10)
        + 2;
    let ip_length = vms
        .iter()
        .map(|vm| vm.ip_address.clone().unwrap_or_default().len())
        .max()
        .unwrap_or(10)
        + 2;
    let created_length = vms
        .iter()
        .map(|vm| format_duration_ago(vm.created_at).len())
        .max()
        .unwrap_or(10)
        + 2;

    println!(
        "{:<name_length$} {:<distro_length$} {:<vcpu_length$} {:<memory_length$} {:<status_length$} {:<pid_length$} {:<ip_length$} {:<created_length$}",
        "NAME", "DISTRO", "VCPU", "MEMORY", "STATUS", "PID", "IP", "CREATED"
    );
    for vm in vms {
        println!(
            "{:<name_length$} {:<distro_length$} {:<vcpu_length$} {:<memory_length$} {:<status_length$} {:<pid_length$} {:<ip_length$} {:<created_length$}",
            vm.name,
            vm.distro,
            vm.vcpu,
            format!("{} MiB", vm.memory),
            format_status(&vm.status, vm.updated_at),
            vm.pid.unwrap_or(0),
            vm.ip_address.unwrap_or_default(),
            format_duration_ago(vm.created_at),
        );
    }

    Ok(())
}

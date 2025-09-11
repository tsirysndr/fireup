use anyhow::Error;

use crate::date::format_duration_ago;

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

    println!(
        "{:<20} {:<10} {:<5} {:<10} {:<15} {:<10} {:<15} {:<10}",
        "NAME", "DISTRO", "VCPU", "MEMORY", "STATUS", "PID", "IP", "CREATED"
    );
    for vm in vms {
        println!(
            "{:<20} {:<10} {:<5} {:<10} {:<15} {:<10} {:<15} {:<10}",
            vm.name,
            vm.distro,
            vm.vcpu,
            format!("{} MiB", vm.memory),
            vm.status,
            vm.pid.unwrap_or(0),
            vm.ip_address.unwrap_or_default(),
            format_duration_ago(vm.created_at),
        );
    }

    Ok(())
}

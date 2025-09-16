use std::process;

use anyhow::Error;
use firecracker_state::repo;

pub async fn remove(name: &str) -> Result<(), Error> {
    let pool = firecracker_state::create_connection_pool().await?;
    let vm = repo::virtual_machine::find(&pool, name).await?;

    if vm.is_none() {
        println!("[!] No virtual machine found with the name: {}", name);
        process::exit(1);
    }
    let vm = vm.unwrap();

    firecracker_process::stop(Some(vm.name)).await.ok();
    repo::virtual_machine::delete(&pool, &vm.id).await?;

    println!("{}", vm.id);

    Ok(())
}

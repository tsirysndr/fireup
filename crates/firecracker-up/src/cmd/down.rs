use anyhow::Error;
use firecracker_vm::types::VmOptions;

pub fn down(options: &VmOptions) -> Result<(), Error> {
    firecracker_process::stop(options)?;
    Ok(())
}

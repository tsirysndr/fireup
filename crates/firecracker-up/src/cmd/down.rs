use anyhow::Error;

pub fn down() -> Result<(), Error> {
    firecracker_process::stop()?;
    Ok(())
}

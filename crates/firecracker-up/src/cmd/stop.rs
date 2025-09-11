use anyhow::Error;

pub async fn stop(name: &str) -> Result<(), Error> {
    firecracker_process::stop(Some(name.to_string())).await?;
    Ok(())
}

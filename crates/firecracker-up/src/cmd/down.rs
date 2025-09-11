use anyhow::Error;

pub async fn down() -> Result<(), Error> {
    firecracker_process::stop(None).await?;
    Ok(())
}

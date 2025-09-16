use anyhow::Error;
use fire_server::server;

pub async fn serve() -> Result<(), Error> {
    server::run().await?;
    Ok(())
}

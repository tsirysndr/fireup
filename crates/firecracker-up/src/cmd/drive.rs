use anyhow::Error;

pub async fn create(name: &str, path_on_host: Option<String>) -> Result<(), Error> {
    Ok(())
}

pub async fn remove(name: &str) -> Result<(), Error> {
    Ok(())
}

pub async fn list() -> Result<(), Error> {
    Ok(())
}

pub async fn attach(name: &str, path_on_host: String) -> Result<(), Error> {
    Ok(())
}

pub async fn inspect(name: &str) -> Result<(), Error> {
    Ok(())
}

pub async fn detach(name: &str, path_on_host: String) -> Result<(), Error> {
    Ok(())
}
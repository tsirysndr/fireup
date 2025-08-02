use anyhow::Error;
use fire_config::init_config;
use owo_colors::OwoColorize;

pub fn init() -> Result<(), Error> {
    init_config()?;
    println!(
        "[+] Firecracker MicroVM configuration initialized successfully: {} created 🎉",
        "`fire.toml`".cyan()
    );
    println!("[✓] Start your MicroVM by running: {}", "fireup".green());
    Ok(())
}

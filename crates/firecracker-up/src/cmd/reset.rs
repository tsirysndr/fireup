use anyhow::Error;
use firecracker_process::stop;
use firecracker_vm::types::VmOptions;
use glob::glob;
use owo_colors::OwoColorize;

pub async fn reset(options: VmOptions) -> Result<(), Error> {
    println!(
        "Are you sure you want to reset? This will remove all ext4 files. Type '{}' to confirm:",
        "yes".bright_green()
    );
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| Error::msg(format!("Failed to read input: {}", e)))?;
    let input = input.trim();

    if input != "yes" {
        println!("Reset cancelled.");
        return Ok(());
    }

    let name = options
        .api_socket
        .trim_start_matches("/tmp/firecracker-")
        .trim_end_matches(".sock")
        .to_string();

    stop(Some(name)).await?;

    let app_dir = crate::config::get_config_dir()?;
    let ext4_file = glob(format!("{}/*.ext4", app_dir).as_str())
        .map_err(|e| Error::msg(format!("Failed to find ext4 file: {}", e)))?;

    for file in ext4_file {
        if let Ok(path) = file {
            std::fs::remove_file(path)
                .map_err(|e| Error::msg(format!("Failed to remove file: {}", e)))?;
        }
    }

    println!("[+] Reset complete. All ext4 files have been removed.");
    println!(
        "[+] You can now run '{}' to start a new Firecracker MicroVM.",
        "fireup".bright_green()
    );

    Ok(())
}

use std::process;

use anyhow::Error;
use firecracker_process::stop;
use firecracker_state::repo;
use firecracker_vm::types::VmOptions;
use glob::glob;
use owo_colors::OwoColorize;

use crate::command::run_command;

pub async fn reset(options: VmOptions) -> Result<(), Error> {
    let name = options
        .api_socket
        .trim_start_matches("/tmp/firecracker-")
        .trim_end_matches(".sock")
        .to_string();

    if options.api_socket.is_empty() {
        println!(
            "Are you sure you want to reset? This will remove all *.img files. Type '{}' to confirm:",
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

        stop(Some(name)).await?;

        let app_dir = crate::config::get_config_dir()?;
        let img_file = glob(format!("{}/*.img", app_dir).as_str())
            .map_err(|e| Error::msg(format!("Failed to find img file: {}", e)))?;

        for file in img_file {
            if let Ok(path) = file {
                run_command("rm", &[path.to_str().unwrap_or_default()], true)?;
            }
        }

        println!("[+] Reset complete. All *.img files have been removed.");
        println!(
            "[+] You can now run '{}' to start a new Firecracker MicroVM.",
            "fireup".bright_green()
        );
        return Ok(());
    }

    println!("Are you sure you want to reset the VM {}? This will remove its associated {} file. Type '{}' to confirm:", name.cyan(), "*.img".cyan(), "yes".bright_green());
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| Error::msg(format!("Failed to read input: {}", e)))?;
    let input = input.trim();
    if input != "yes" {
        println!("Reset cancelled.");
        return Ok(());
    }

    let pool = firecracker_state::create_connection_pool().await?;

    let vm = repo::virtual_machine::find_by_api_socket(&pool, &options.api_socket).await?;
    if vm.is_none() {
        println!("[!] No virtual machine found with name: {}", name);
        process::exit(1);
    }

    let vm = vm.unwrap();
    stop(Some(vm.name.clone())).await?;

    if let Some(rootfs) = &vm.rootfs {
        run_command("rm", &[rootfs], true)?;
    }

    println!("[+] Reset complete. Associated *.img files have been removed.");
    println!(
        "[+] You can now run '{}' to start a new Firecracker MicroVM.",
        "fireup".bright_green()
    );

    Ok(())
}

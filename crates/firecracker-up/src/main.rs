use std::thread;

use anyhow::Result;

fn main() -> Result<()> {
    firecracker_process::start()?;

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
        if firecracker_process::is_running() {
            println!("[+] Firecracker is running.");
            break;
        }
    }

    firecracker_prepare::prepare()?;
    firecracker_vm::setup()?;
    Ok(())
}

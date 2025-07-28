use anyhow::Error;

use crate::command::run_command;

pub fn logs(follow: bool) -> Result<(), Error> {
    let app_dir = crate::config::get_config_dir()?;
    let logfile = format!("{}/firecracker.log", app_dir);
    let logfile = logfile.as_str();
    run_command(
        "tail",
        &match follow {
            true => vec!["-f", logfile],
            false => vec![logfile],
        },
        true,
    )?;
    Ok(())
}

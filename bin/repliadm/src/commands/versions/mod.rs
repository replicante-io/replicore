use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use slog::Logger;
use slog::info;
use slog::warn;

use replicante_util_failure::failure_info;

mod coordinator;
mod replicante;
mod stores;
mod task_queue;

use crate::utils::load_config;
use crate::Interfaces;
use crate::Result;

pub const COMMAND: &str = "versions";

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Report version information for various systems")
        .arg(
            Arg::with_name("cluster")
                .long("cluster")
                .value_name("URL")
                .default_value("http://localhost:16016/")
                .takes_value(true)
                .help("URL of the Replicante Core cluster to connect to"),
        )
}

pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let logger = interfaces.logger();
    info!(logger, "Showing versions");

    let config = load_config(args)?;
    replicante::versions(args, logger)?;
    coordinator::version(&config, logger)?;
    stores::primary(&config, logger)?;
    stores::view(&config, logger)?;
    task_queue::version(&config, logger)?;

    Ok(())
}

/// Returns the value of the result or a formatted error message.
fn value_or_error(logger: &Logger, tool: &'static str, result: Result<String>) -> String {
    match result {
        Err(error) => {
            warn!(logger, "Failed to detect {} version", tool; failure_info(&error));
            error.to_string().trim_end().to_string()
        }
        Ok(value) => value,
    }
}

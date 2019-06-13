use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;
use lazy_static::lazy_static;
use slog::info;
use slog::warn;
use slog::Logger;

use replicante::Config;
use replicante::ReplicanteQueues;
use replicante::VERSION as REPLICANTE_VERSION;
use replicante_data_store::admin::Admin as StoreAdmin;
use replicante_service_coordinator::Admin as CoordinatorAdmin;
use replicante_service_tasks::Admin as TasksAdmin;
use replicante_util_failure::failure_info;

use super::super::core::Client;
use super::super::Error;
use super::super::ErrorKind;
use super::super::Interfaces;
use super::super::Result;

pub const COMMAND: &str = "versions";

lazy_static! {
    /// Version details for replictl.
    static ref VERSION: String = format!(
        "{} [{}; {}]",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_BUILD_HASH"),
        env!("GIT_BUILD_TAINT")
    );
}

/// Configure the `replictl version` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND).about("Reports version information for various systems")
}

/// Report all replicante versions (replictl, static lib, running cluster).
fn replicante_versions<'a>(args: &ArgMatches<'a>, logger: &Logger) -> Result<()> {
    println!("replictl: {}", *VERSION);
    println!(
        "Replicante (statically determined): {}",
        *REPLICANTE_VERSION
    );
    let version = Client::new(args)
        .and_then(|client| client.version())
        .map(|version| {
            format!(
                "{} [{}; {}]",
                version.version, version.commit, version.taint
            )
        });
    println!(
        "Replicante (dynamically determined): {}",
        value_or_error(logger, "replicante dynamic", version)
    );
    Ok(())
}

/// Switch the control flow to the requested check command.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let logger = interfaces.logger();
    info!(logger, "Showing versions");

    // Find external systems version.
    let config = args.value_of("config").unwrap();
    let config = Config::from_file(config).context(ErrorKind::ConfigLoad)?;

    replicante_versions(args, logger)?;
    coordinator_version(&config, logger)?;
    primary_store_version(&config, logger)?;
    task_queue_version(&config, logger)?;
    Ok(())
}

/// Collect version information for the configured coordinator.
fn coordinator_version(config: &Config, logger: &Logger) -> Result<()> {
    let version = CoordinatorAdmin::new(config.coordinator.clone(), logger.clone())
        .with_context(|_| ErrorKind::AdminInit("coordinator"))
        .and_then(|admin| {
            admin
                .version()
                .with_context(|_| ErrorKind::FetchVersion("coordinator"))
        })
        .map_err(Error::from);
    println!(
        "Coordinator: {}",
        value_or_error(logger, "coordinator", version)
    );
    Ok(())
}

/// Collect version information for the configured primary store.
fn primary_store_version(config: &Config, logger: &Logger) -> Result<()> {
    let version = StoreAdmin::make(config.storage.clone(), logger.clone())
        .with_context(|_| ErrorKind::AdminInit("primary store"))
        .and_then(|store| {
            store
                .version()
                .with_context(|_| ErrorKind::FetchVersion("store"))
        })
        .map_err(Error::from)
        .map(|v| format!("{} {}", v.tag, v.version));
    println!(
        "Primary Store: {}",
        value_or_error(logger, "primary store", version)
    );
    Ok(())
}

/// Collect version information for the configured tasks queue.
fn task_queue_version(config: &Config, logger: &Logger) -> Result<()> {
    let version = TasksAdmin::<ReplicanteQueues>::new(logger.clone(), config.tasks.clone())
        .with_context(|_| ErrorKind::AdminInit("tasks queue"))
        .and_then(|tasks| {
            tasks
                .version()
                .with_context(|_| ErrorKind::FetchVersion("tasks queue"))
        })
        .map_err(Error::from);
    println!(
        "Tasks Queue: {}",
        value_or_error(logger, "tasks queue", version)
    );
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

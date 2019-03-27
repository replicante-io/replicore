extern crate bodyparser;
extern crate chrono;
extern crate clap;
#[macro_use]
extern crate failure;
extern crate failure_derive;

extern crate iron;
extern crate iron_json_response;
extern crate router;
#[cfg(test)]
extern crate iron_test;

#[macro_use]
extern crate lazy_static;

extern crate opentracingrust;
extern crate prometheus;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;

#[macro_use]
extern crate slog;
extern crate slog_scope;
extern crate slog_stdlog;

extern crate replicante_agent_client;
extern crate replicante_agent_discovery;
extern crate replicante_agent_models;
extern crate replicante_coordinator;
extern crate replicante_data_aggregator;
extern crate replicante_data_fetcher;
extern crate replicante_data_models;
extern crate replicante_data_store;
extern crate replicante_logging;
extern crate replicante_streams_events;
extern crate replicante_tasks;
extern crate replicante_util_failure;
extern crate replicante_util_iron;
extern crate replicante_util_tracing;


use clap::App;
use clap::Arg;
use failure::ResultExt;
use slog::Logger;


mod components;
mod config;
mod error;
mod interfaces;
mod metrics;
mod tasks;

use self::components::Components;
use self::interfaces::Interfaces;

pub use self::config::Config;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::tasks::ReplicanteQueues;
pub use self::tasks::payload as task_payload;


lazy_static! {
    /// Version details for replictl.
    pub static ref VERSION: String = format!(
        "{} [{}; {}]",
        env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_HASH"), env!("GIT_BUILD_TAINT")
    );
}


/// Initialised interfaces and components and waits for the system to exit.
///
/// Replicante is built on top of two kinds of units:
///
///   * Interfaces: units used to inspect the system or interact with it.
///   * Components: units that perfom actions and implement logic.
///
/// Most, if not all, components start background threads and must join on drop.
/// Interfaces can work in the same way if they need threads but some may just provide
/// services to other interfaces and/or components.
#[allow(clippy::needless_pass_by_value)]
fn initialise_and_run(config: Config, logger: Logger) -> Result<()> {
    // TODO: iniialise sentry as soon as possible.

    info!(logger, "Initialising sub-systems ...");
    // Need to initialise the interfaces before we can register all metrics.
    let mut interfaces = Interfaces::new(&config, logger.clone())?;
    Interfaces::register_metrics(&logger, interfaces.metrics.registry());
    Components::register_metrics(&logger, interfaces.metrics.registry());
    self::metrics::register_metrics(&logger, interfaces.metrics.registry());
    let mut components = Components::new(&config, logger.clone(), &mut interfaces)?;

    // Initialisation done, run all interfaces and components.
    info!(logger, "Starting sub-systems ...");
    interfaces.run()?;
    components.run()?;

    // Wait for interfaces and components to terminate.
    info!(logger, "Replicante ready");
    interfaces.wait_all()?;
    components.wait_all()?;

    info!(logger, "Replicante stopped gracefully");
    Ok(())
}


/// Parse command line, load configuration, initialise logger.
///
/// Once the configuration is loaded control is passed to `initialise_and_run`.
pub fn run() -> Result<()> {
    // Initialise and parse command line arguments.
    let version = format!(
        "{} [{}; {}]",
        env!("CARGO_PKG_VERSION"), env!("GIT_BUILD_HASH"), env!("GIT_BUILD_TAINT")
    );
    let cli_args = App::new("Replicante Core")
        .version(version.as_ref())
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .default_value("replicante.yaml")
             .help("Specifies the configuration file to use")
             .takes_value(true)
        )
        .get_matches();

    // Log initialisation start message.
    let logger_opts = replicante_logging::Opts::new(env!("GIT_BUILD_HASH").into());
    let logger = replicante_logging::starter(&logger_opts);
    info!(logger, "Starting replicante core"; "git-taint" => env!("GIT_BUILD_TAINT"));

    // Load configuration.
    let config_location = cli_args.value_of("config").unwrap();
    info!(logger, "Loading configuration ..."; "config" => config_location);
    let config = Config::from_file(config_location).context(
        ErrorKind::Legacy(format_err!("failed to load configuration: {}", config_location))
    )?;
    let config = config.transform();

    // Initialise and run forever.
    let logger = replicante_logging::configure(config.logging.clone(), &logger_opts);
    let _scope_guard = slog_scope::set_global_logger(logger.clone());
    slog_stdlog::init().expect("Failed to initialise log -> slog integration");
    debug!(logger, "Logging configured");

    let result = initialise_and_run(config, logger.clone());
    warn!(logger, "Shutdown: system exiting now"; "error" => result.is_err());
    result
}

extern crate clap;

#[macro_use]
extern crate error_chain;

extern crate iron;
extern crate iron_json_response;
extern crate router;
#[cfg(test)]
extern crate iron_test;

#[macro_use]
extern crate lazy_static;

extern crate opentracingrust;
extern crate prometheus;

extern crate replicante_agent_client;
extern crate replicante_agent_discovery;
extern crate replicante_data_models;
extern crate replicante_data_store;
extern crate replicante_util_iron;
extern crate replicante_util_tracing;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

#[macro_use]
extern crate slog;
extern crate slog_async;
#[cfg(feature = "journald")]
extern crate slog_journald;
extern crate slog_json;

use clap::App;
use clap::Arg;
use slog::Logger;


mod components;
mod config;
mod errors;
mod interfaces;
mod logging;

use self::components::Components;
use self::config::Config;
use self::interfaces::Interfaces;

pub use self::errors::Error;
pub use self::errors::ErrorKind;
pub use self::errors::ResultExt;
pub use self::errors::Result;


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
fn initialise_and_run(config: Config, logger: Logger) -> Result<()> {
    info!(logger, "Initialising sub-systems ...");
    let mut interfaces = Interfaces::new(&config, logger.clone())?;
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
        .version(&version[..])
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
    let logger = logging::starter();
    info!(logger, "Starting replicante core"; "git-taint" => env!("GIT_BUILD_TAINT"));

    // Load configuration.
    let config_location = cli_args.value_of("config").unwrap();
    info!(logger, "Loading configuration ..."; "config" => config_location);
    let config = Config::from_file(config_location.clone())
        .chain_err(|| format!("Failed to load configuration: {}", config_location))?;

    // Initialise and run forever.
    let logger = logging::configure(config.logging.clone());
    debug!(logger, "Logging configured");

    let result = initialise_and_run(config, logger.clone());
    warn!(logger, "Shutdown: system exiting now"; "error" => result.is_err());
    result
}

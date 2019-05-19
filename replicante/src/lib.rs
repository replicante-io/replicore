extern crate bodyparser;
extern crate chrono;
extern crate clap;
extern crate failure;
extern crate humthreads;
extern crate iron;
extern crate iron_json_response;
#[cfg(test)]
extern crate iron_test;
extern crate router;
#[macro_use]
extern crate lazy_static;
extern crate opentracingrust;
extern crate prometheus;
extern crate sentry;
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
extern crate replicante_util_upkeep;

use clap::App;
use clap::Arg;
use failure::ResultExt;
use sentry::integrations::failure::capture_fail;
use sentry::internals::ClientInitGuard;
use sentry::internals::IntoDsn;
use slog::Logger;

use replicante_util_upkeep::Upkeep;

mod components;
mod config;
mod error;
mod interfaces;
mod metrics;
mod tasks;

pub use self::config::Config;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::tasks::payload as task_payload;
pub use self::tasks::ReplicanteQueues;

use self::components::Components;
use self::config::SentryConfig;
use self::interfaces::Interfaces;

lazy_static! {
    static ref RELEASE: String = format!("replicore@{}", env!("GIT_BUILD_HASH"));
    pub static ref VERSION: String = format!(
        "{} [{}; {}]",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_BUILD_HASH"),
        env!("GIT_BUILD_TAINT"),
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
fn initialise_and_run(config: Config, logger: Logger) -> Result<bool> {
    // Initialise Upkeep instance and signals.
    let mut upkeep = Upkeep::new();
    upkeep
        .register_signal()
        .with_context(|_| ErrorKind::InterfaceInit("UNIX signal"))?;
    upkeep.set_logger(logger.clone());

    // Need to initialise the interfaces before we can register all metrics.
    info!(logger, "Initialising sub-systems ...");
    let mut interfaces = Interfaces::new(&config, logger.clone(), &mut upkeep)?;
    Interfaces::register_metrics(&logger, interfaces.metrics.registry());
    Components::register_metrics(&logger, interfaces.metrics.registry());
    self::metrics::register_metrics(&logger, interfaces.metrics.registry());
    let mut components = Components::new(&config, logger.clone(), &mut interfaces)?;

    // Initialisation done, run all interfaces and components.
    info!(logger, "Starting sub-systems ...");
    interfaces.run(&mut upkeep)?;
    components.run(&mut upkeep)?;

    // Wait for interfaces and components to terminate.
    info!(logger, "Replicante is ready");
    let clean_exit = upkeep.keepalive();
    if clean_exit {
        info!(logger, "Replicante stopped gracefully");
    } else {
        warn!(logger, "Exiting due to error in a worker thread");
    }
    Ok(clean_exit)
}

/// Initialise sentry integration.
///
/// If sentry is configured, the panic handler is also registered.
pub fn initialise_sentry(config: Option<SentryConfig>, logger: &Logger) -> Result<ClientInitGuard> {
    let config = match config {
        None => {
            info!(logger, "Not using sentry: no configuration provided");
            return Ok(sentry::init(()));
        }
        Some(config) => config,
    };
    info!(logger, "Configuring sentry integration");
    let dsn = config
        .dsn
        .into_dsn()
        .with_context(|_| ErrorKind::InterfaceInit("sentry"))?;
    let client = sentry::init(sentry::ClientOptions {
        attach_stacktrace: true,
        dsn,
        in_app_include: vec!["replicante"],
        release: Some(RELEASE.as_str().into()),
        ..Default::default()
    });
    if client.is_enabled() {
        sentry::integrations::panic::register_panic_handler();
    }
    Ok(client)
}

/// Parse command line, load configuration, initialise logger.
///
/// Once the configuration is loaded control is passed to `initialise_and_run`.
pub fn run() -> Result<bool> {
    // Initialise and parse command line arguments.
    let version = format!(
        "{} [{}; {}]",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_BUILD_HASH"),
        env!("GIT_BUILD_TAINT")
    );
    let cli_args = App::new("Replicante Core")
        .version(version.as_ref())
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .default_value("replicante.yaml")
                .help("Specifies the configuration file to use")
                .takes_value(true),
        )
        .get_matches();

    // Log initialisation start message.
    let logger_opts = replicante_logging::Opts::new(env!("GIT_BUILD_HASH").into());
    let logger = replicante_logging::starter(&logger_opts);
    info!(logger, "Starting replicante core"; "git-taint" => env!("GIT_BUILD_TAINT"));

    // Load configuration.
    let config_location = cli_args.value_of("config").unwrap();
    info!(logger, "Loading configuration ..."; "config" => config_location);
    let config = Config::from_file(config_location).with_context(|_| ErrorKind::ConfigLoad)?;
    let config = config.transform();

    // Initialise and run forever.
    let logger = replicante_logging::configure(config.logging.clone(), &logger_opts);
    let _scope_guard = slog_scope::set_global_logger(logger.clone());
    slog_stdlog::init().expect("Failed to initialise log -> slog integration");
    debug!(logger, "Logging configured");

    // Iniialise sentry as soon as possible.
    let _sentry = initialise_sentry(config.sentry.clone(), &logger)?;
    let result = initialise_and_run(config, logger.clone()).map_err(|error| {
        capture_fail(&error);
        error
    });
    let error = match &result {
        Err(_) => true,
        Ok(clean) => *clean,
    };
    warn!(logger, "Shutdown: system exiting now"; "error" => error);
    result
}

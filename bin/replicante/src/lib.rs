use clap::Parser;
use failure::ResultExt;
use prometheus::Registry;
use sentry::integrations::anyhow::capture_anyhow;
use sentry::ClientInitGuard;
use sentry::IntoDsn;
use slog::debug;
use slog::info;
use slog::warn;
use slog::Logger;

use replicante_util_upkeep::Upkeep;

use replicore_iface_orchestrator_action::OrchestratorActionRegistryBuilder;

mod components;
mod config;
mod error;
mod interfaces;
mod metrics;

pub use self::config::Config;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;

use self::components::Components;
use self::config::SentryConfig;
use self::interfaces::Interfaces;

const RELEASE: &str = concat!("replicore@", env!("GIT_BUILD_HASH"));
pub const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " [",
    env!("GIT_BUILD_HASH"),
    "; ",
    env!("GIT_BUILD_TAINT"),
    "]",
);

/// Replicante Core datastore orchestration control plane.
#[derive(Debug, Parser)]
#[command(long_about = None)]
#[command(version = VERSION)]
pub struct Cli {
    /// Specifies the configuration file to use.
    #[arg(short = 'c', long = "config", value_name = "FILE")]
    #[arg(default_value = "replicante.yaml")]
    config: String,
}

/// Initialised interfaces and components and waits for the system to exit.
///
/// Replicante is built on top of two kinds of units:
///
///   * Interfaces: units used to inspect the system or interact with it.
///   * Components: units that perform actions and implement logic.
///
/// Most, if not all, components start background threads and must join on drop.
/// Interfaces can work in the same way if they need threads but some may just provide
/// services to other interfaces and/or components.
#[allow(clippy::needless_pass_by_value)]
fn initialise_and_run(config: Config, logger: Logger) -> Result<bool> {
    // Register built-in actions before any other thread is spawned.
    #[allow(unused_mut)]
    let mut builder = OrchestratorActionRegistryBuilder::empty();

    #[cfg(feature = "action-debug")]
    replicore_action_debug::register(&mut builder)
        .map_err(replicore_util_errors::AnyWrap::from)
        .with_context(|_| ErrorKind::InterfaceInit("orchestrator actions registry"))?;
    #[cfg(feature = "action-http")]
    replicore_action_http::register(&mut builder)
        .map_err(replicore_util_errors::AnyWrap::from)
        .with_context(|_| ErrorKind::InterfaceInit("orchestrator actions registry"))?;

    builder.build_as_current();

    // Initialise Upkeep instance and signals.
    let mut upkeep = Upkeep::new();
    upkeep
        .register_signal()
        .with_context(|_| ErrorKind::InterfaceInit("UNIX signal"))?;
    upkeep.set_logger(logger.clone());

    // Need to initialise the interfaces before we can register all metrics.
    info!(logger, "Initialising sub-systems ...");
    let mut interfaces = Interfaces::new(&config, logger.clone(), &mut upkeep)?;
    register_crates_metrics(&logger, interfaces.metrics.registry());
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
    let options = sentry::ClientOptions {
        attach_stacktrace: true,
        dsn,
        in_app_include: vec!["replicante", "replicore", "replisdk"],
        release: Some(RELEASE.into()),
        ..Default::default()
    };
    let client = sentry::init(options);
    Ok(client)
}

/// Attempt to register all metrics from other replicante_* crates.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_crates_metrics(logger: &Logger, registry: &Registry) {
    replicante_agent_client::register_metrics(logger, registry);
    replicante_cluster_discovery::register_metrics(logger, registry);
    replicante_externals_kafka::register_metrics(logger, registry);
    replicante_externals_mongodb::register_metrics(logger, registry);
    replicante_service_coordinator::register_metrics(logger, registry);
    replicante_service_tasks::register_metrics(logger, registry);
    replicante_stream::register_metrics(logger, registry);
    replicore_cluster_orchestrate::register_metrics(logger, registry);
}

/// Parse command line, load configuration, initialise logger.
///
/// Once the configuration is loaded control is passed to `initialise_and_run`.
pub fn run() -> Result<bool> {
    // Initialise and parse command line arguments.
    let args = Cli::parse();

    // Log initialisation start message.
    let logger_opts = replicante_logging::Opts::new(env!("GIT_BUILD_HASH").into());
    let logger = replicante_logging::starter(&logger_opts);
    info!(
        logger,
        "Starting replicante core";
        "git-hash" => env!("GIT_BUILD_HASH"),
        "git-taint" => env!("GIT_BUILD_TAINT"),
        "version" => env!("CARGO_PKG_VERSION"),
    );

    // Load configuration.
    let config_location = &args.config;
    info!(logger, "Loading configuration ..."; "config" => config_location);
    let config = Config::from_file(config_location).with_context(|_| ErrorKind::ConfigLoad)?;
    let config = config.transform();

    // Initialise and run forever.
    let logger = replicante_logging::configure(config.logging.clone(), &logger_opts);
    let _scope_guard = slog_scope::set_global_logger(logger.clone());
    slog_stdlog::init().expect("Failed to initialise log -> slog integration");
    debug!(logger, "Logging configured");

    // Initialise sentry as soon as possible.
    let _sentry = initialise_sentry(config.sentry.clone(), &logger)?;
    let result = initialise_and_run(config, logger.clone()).map_err(|error| {
        // TODO: Fix error capturing after failure crate is removed.
        let hack = anyhow::anyhow!(error.to_string());
        capture_anyhow(&hack);
        error
    });
    let error = match &result {
        Err(_) => true,
        Ok(clean) => !*clean,
    };
    warn!(logger, "Shutdown: system exiting now"; "error" => error);
    result
}

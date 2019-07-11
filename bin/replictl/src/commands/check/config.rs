use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;
use slog::error;
use slog::info;
use slog::warn;

use replicante::Config;

use crate::outcome::Outcomes;
use crate::outcome::Warning;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub const COMMAND: &str = "config";
const DISCOVERY_INTERVAL_THRESHOLD: u64 = 15;

/// Configure the `replictl check config` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND).about("Check the replicante configuration for errors")
}

/// Check the replicante configuration for errors.
///
/// The following checks are performed:
///
///   * Replicante core configuration loads.
///   * File discovery files load.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let file = args.value_of("config").unwrap();
    let logger = interfaces.logger();
    info!(logger, "Checking configuration"; "file" => file);

    // Load core config.
    let config = Config::from_file(file).with_context(|_| ErrorKind::ConfigLoad)?;

    // Core config checks.
    let mut outcomes = Outcomes::new();
    if config.discovery.interval < DISCOVERY_INTERVAL_THRESHOLD {
        outcomes.warn(Warning::BelowThreshold(
            "'discovery.interval' is very frequent".into(),
            config.discovery.interval,
            DISCOVERY_INTERVAL_THRESHOLD,
        ));
    }

    // Report results.
    outcomes.report(&logger);
    if outcomes.has_errors() {
        error!(logger, "Configuration checks failed");
        return Err(ErrorKind::CheckWithErrors("configuration").into());
    }
    if outcomes.has_warnings() {
        warn!(logger, "Configuration checks passed with warnings");
        return Ok(());
    }
    info!(logger, "Configuration checks passed");
    Ok(())
}

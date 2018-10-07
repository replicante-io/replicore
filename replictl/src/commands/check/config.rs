use std::fs::File;

use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use error_chain::ChainedError;
use serde_yaml;

use replicante::Config;
use replicante_agent_discovery::DiscoveryFileModel;

use super::super::super::Interfaces;
use super::super::super::Result;
use super::super::super::ResultExt;

use super::super::super::outcome::Error;
use super::super::super::outcome::Outcomes;
use super::super::super::outcome::Warning;


pub const COMMAND: &str = "config";
const DISCOVERY_INTERVAL_THRESHOLD: u64 = 15;


/// Configure the `replictl check config` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Check the replicante configuration for errors")
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
    let config = match Config::from_file(file) {
        Ok(config) => config,
        Err(error) => {
            let error_string = error.display_chain().to_string();
            error!(logger, "Configuration checks failed"; "error" => error_string);
            return Err(error).chain_err(|| "Check failed: could not load configuration");
        }
    };

    // Core config checks.
    let mut outcomes = Outcomes::new();
    if config.discovery.interval < DISCOVERY_INTERVAL_THRESHOLD {
        outcomes.warn(Warning::BelowThreshold(
            "'discovery.interval' is very frequent".into(),
            config.discovery.interval, DISCOVERY_INTERVAL_THRESHOLD
        ));
    }
    outcomes.report(&logger);

    // Check each file discovery config.
    let mut tracker = interfaces.progress("Processed more file discovery configurations");
    for file in config.discovery.backends.files.iter() {
        check_discovery_file(file, &mut outcomes);
        tracker.track();
    }

    // Report results.
    outcomes.report(&logger);
    if outcomes.has_errors() {
        error!(logger, "Configuration checks failed");
        return Err("Configuration checks failed".into());
    }
    if outcomes.has_warnings() {
        warn!(logger, "Configuration checks passed with warnings");
        return Ok(());
    }
    info!(logger, "Configuration checks passed");
    Ok(())
}


/// Attempt to load and parse file.
fn check_discovery_file(path: &str, outcomes: &mut Outcomes) {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            let error: super::super::super::Error = error.into();
            let error = error.chain_err(
                || format!("Failed to open file discovery unit: {}", path)
            );
            let error = error.display_chain().to_string();
            outcomes.error(Error::GenericError(error));
            return;
        }
    };
    let _content: DiscoveryFileModel = match serde_yaml::from_reader(file) {
        Ok(content) => content,
        Err(error) => {
            let error: super::super::super::Error = error.into();
            let error = error.display_chain().to_string();
            outcomes.error(Error::UnableToParseModel(
                "DiscoveryFile".into(), path.to_string(), error
            ));
            return;
        }
    };
}

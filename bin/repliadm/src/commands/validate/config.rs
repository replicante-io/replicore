use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;
use slog::info;

use replicante::Config;
use replicante_util_failure::format_fail;

pub const COMMAND: &str = "config";
const DISCOVERY_INTERVAL_THRESHOLD: u64 = 15;

use crate::outcome::Error;
use crate::outcome::Outcomes;
use crate::outcome::Warning;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND).about("Validate Replicante Core configuration")
}

pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<Outcomes> {
    let logger = interfaces.logger();
    let file = args
        .value_of("config")
        .expect("CLI argument --config is required");
    info!(logger, "Checking configuration"; "file" => file);

    // Load core config.
    let mut outcomes = Outcomes::new();
    let config = match Config::from_file(file).with_context(|_| ErrorKind::ConfigLoad) {
        Ok(config) => config,
        Err(error) => {
            let error = format_fail(&error);
            outcomes.error(Error::Generic(error));
            return Ok(outcomes);
        }
    };

    // Core config checks.
    if config.discovery.interval < DISCOVERY_INTERVAL_THRESHOLD {
        outcomes.warn(Warning::BelowThreshold(
            "'discovery.interval' is very frequent".into(),
            config.discovery.interval,
            DISCOVERY_INTERVAL_THRESHOLD,
        ));
    }
    Ok(outcomes)
}

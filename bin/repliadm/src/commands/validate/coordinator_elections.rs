use clap::ArgMatches;
use clap::Command;
use slog::info;

use replicante_util_failure::format_fail;

pub const COMMAND: &str = "coordinator-elections";

use crate::outcome::Error;
use crate::outcome::Outcomes;
use crate::utils::coordinator_admin;
use crate::Interfaces;
use crate::Result;

pub fn command() -> Command {
    Command::new(COMMAND).about("Validate coordinator elections")
}

pub fn run(args: &ArgMatches, interfaces: &Interfaces) -> Result<Outcomes> {
    let logger = interfaces.logger();
    info!(logger, "Checking active elections");

    let admin = coordinator_admin(args, logger.clone())?;
    let mut outcomes = Outcomes::new();
    let mut tracker = interfaces.progress("Processed more elections");

    for election in admin.elections() {
        if let Err(error) = election {
            let error = format_fail(&error);
            outcomes.error(Error::Generic(error));
        }
        tracker.track();
    }

    Ok(outcomes)
}

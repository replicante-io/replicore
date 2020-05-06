use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use failure::Fail;
use slog::debug;
use slog::info;

use replicante_externals_mongodb::admin::ValidationResult;
use replicante_util_failure::format_fail;

pub const COMMAND: &str = "primary-store-schema";

use crate::outcome::Error;
use crate::outcome::Outcomes;
use crate::outcome::Warning;
use crate::utils::primary_store_admin;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND).about("Validate the primary store schema")
}

/// Validate the primary store schema compatibility with this version of replicante.
///
/// The following checks are performed:
///
///   * All needed collections/tables exist and have the correct schema.
///   * All needed and recommended indexes exist.
///   * No dropped collections/tables or indexes exist.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<Outcomes> {
    let logger = interfaces.logger();
    let store = primary_store_admin(args, logger.clone())?;
    let mut outcomes = Outcomes::new();
    info!(logger, "Checking primary store schema");

    debug!(logger, "Checking primary schema");
    match store.validate().schema() {
        Ok(results) => consume_results(results, &mut outcomes),
        Err(error) => {
            let error = error.context(ErrorKind::ValidationError("current schema"));
            outcomes.error(Error::GenericError(format_fail(&error)));
        }
    };
    outcomes.report(&logger);

    debug!(
        logger,
        "Checking primary store for removed collections/tables or indexes",
    );
    match store.validate().removed_entities() {
        Ok(results) => consume_results(results, &mut outcomes),
        Err(error) => {
            let error = error.context(ErrorKind::ValidationError("removed collections or indexes"));
            outcomes.error(Error::GenericError(format_fail(&error)));
        }
    };
    outcomes.report(&logger);

    Ok(outcomes)
}

fn consume_results(results: Vec<ValidationResult>, outcomes: &mut Outcomes) {
    for result in results {
        if result.error {
            outcomes.error(Error::StoreValidationError(result));
        } else {
            outcomes.warn(Warning::StoreValidationWarning(result));
        }
    }
}

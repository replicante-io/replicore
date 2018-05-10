use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use error_chain::ChainedError;
use prometheus::Registry;

use replicante::Config;
use replicante_data_store::ValidationResult;
use replicante_data_store::Validator;

use super::super::super::Interfaces;
use super::super::super::Result;
use super::super::super::ResultExt;

use super::super::super::outcome::Error;
use super::super::super::outcome::Outcomes;
use super::super::super::outcome::Warning;


pub const COMMAND: &'static str = "store";

const SCHEMA_COMMAND: &'static str = "schema";
const FAILED_CHECK_SCHEMA : &'static str = "Failed to check store schema";


/// Configure the `replictl check store` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Check the primary store for incompatibilities")
        .subcommand(SubCommand::with_name(SCHEMA_COMMAND)
            .about("Check the primary store schema compatibility with this version of replicante")
        )
}


/// Check the primary store for incompatibilities.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name().clone();
    match command {
        Some(SCHEMA_COMMAND) => schema(args, interfaces),
        None => Err("Need a store check to run".into()),
        _ => Err("Received unrecognised command".into()),
    }
}


/// Check the primary store schema compatibility with this version of replicante.
///
/// The following checks are performed:
///
///   * All needed collections/tables exist and have the correct schema.
///   * All needed and recommended indexes exist.
///   * No dropped collections/tables or indexes exist.
pub fn schema<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let logger = interfaces.logger();
    info!(logger, "Checking store schema");

    let config = args.value_of("config").unwrap();
    let config = Config::from_file(config).chain_err(|| FAILED_CHECK_SCHEMA)?;
    let registry = Registry::new();
    let store = Validator::new(config.storage, logger.clone(), &registry)
        .chain_err(|| FAILED_CHECK_SCHEMA)?;
    let mut outcomes = Outcomes::new();

    debug!(logger, "Checking schema");
    match store.schema() {
        Ok(results) => consume_results(results, &mut outcomes),
        Err(error) => {
            let error = error.chain_err(|| "Failed to validate store schema")
                .display_chain().to_string();
            outcomes.error(Error::GenericError(error));
        }
    };
    outcomes.report(&logger);

    debug!(logger, "Checking indexes");
    match store.indexes() {
        Ok(results) => consume_results(results, &mut outcomes),
        Err(error) => {
            let error = error.chain_err(|| "Failed to validate store indexes")
                .display_chain().to_string();
            outcomes.error(Error::GenericError(error));
        }
    };
    outcomes.report(&logger);

    debug!(logger, "Checking for removed collections/tables or indexes");
    match store.removed() {
        Ok(results) => consume_results(results, &mut outcomes),
        Err(error) => {
            let error = error.chain_err(|| "Failed to check for removed collections or indexes")
                .display_chain().to_string();
            outcomes.error(Error::GenericError(error));
        }
    };
    outcomes.report(&logger);

    // Finish up.
    if outcomes.has_errors() {
        error!(logger, "Store schema checks failed");
        return Err("Store schema checks failed".into());
    }
    if outcomes.has_warnings() {
        warn!(logger, "Store schema checks passed with warnings");
        return Ok(());
    }
    info!(logger, "Store schema checks passed");
    Ok(())
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

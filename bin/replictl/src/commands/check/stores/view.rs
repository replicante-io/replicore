use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use failure::Fail;
use failure::ResultExt;
use slog::debug;
use slog::error;
use slog::info;
use slog::warn;

use replicante::Config;
use replicante_externals_mongodb::admin::ValidationResult;
use replicante_store_view::admin::Admin;
use replicante_store_view::Cursor;
use replicante_util_failure::format_fail;

use crate::outcome::Error;
use crate::outcome::Outcomes;
use crate::outcome::Warning;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub const COMMAND: &str = "view";

const COMMAND_DATA: &str = "data";
const COMMAND_SCHEMA: &str = "schema";
const MODEL_EVENT: &str = "Event";

/// Configure the `replictl check stores view` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Check the view store for incompatibilities")
        .subcommand(SubCommand::with_name(COMMAND_DATA).about(
            "Check ALL view store content for compatibility with this version of replicante",
        ))
        .subcommand(
            SubCommand::with_name(COMMAND_SCHEMA)
                .about("Check the view store schema compatibility with this version of replicante"),
        )
}

/// Check the primary store for incompatibilities.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::super::COMMAND).unwrap();
    let command = command.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name();
    match command {
        Some(COMMAND_DATA) => data(args, interfaces),
        Some(COMMAND_SCHEMA) => schema(args, interfaces),
        None => Err(ErrorKind::NoCommand("replictl check stores view").into()),
        Some(name) => {
            Err(ErrorKind::UnkownSubcommand("replictl check stores view", name.to_string()).into())
        }
    }
}

/// Check ALL view store content for compatibility with this version of replicante.
///
/// The following checks are performed:
///
///   * Each content item is loaded and parsed.
pub fn data<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let logger = interfaces.logger();
    info!(logger, "Checking view store data");
    let confirm = interfaces.prompt().confirm_danger(
        "About to scan ALL content of the view store. \
         This could impact your production system. \
         Would you like to proceed?",
    )?;
    if !confirm {
        error!(logger, "Cannot check without user interactive confirmation");
        return Err(ErrorKind::UserAbort.into());
    }

    let mut outcomes = Outcomes::new();
    let config = args.value_of("config").unwrap();
    let config = Config::from_file(config).with_context(|_| ErrorKind::ConfigLoad)?;
    let admin = Admin::make(config.storage.view.clone(), logger.clone())
        .with_context(|_| ErrorKind::AdminInit("view store"))?;

    info!(logger, "Checking records for the '{}' model", MODEL_EVENT);
    scan_collection(
        admin.data().events(),
        MODEL_EVENT,
        &mut outcomes,
        interfaces,
    );
    outcomes.report(&logger);

    // Report results.
    if outcomes.has_errors() {
        error!(logger, "View store data checks failed");
        return Err(ErrorKind::CheckWithErrors("view store data").into());
    }
    if outcomes.has_warnings() {
        warn!(logger, "View store data checks passed with warnings");
        return Ok(());
    }
    info!(logger, "View store data checks passed");
    Ok(())
}

fn scan_collection<Model>(
    cursor: replicante_store_view::Result<Cursor<Model>>,
    collection: &str,
    outcomes: &mut Outcomes,
    interfaces: &Interfaces,
) where
    Model: ::std::fmt::Debug,
{
    let cursor = match cursor {
        Ok(cursor) => cursor,
        Err(error) => {
            let error = error.to_string();
            outcomes.error(Error::GenericError(error));
            return;
        }
    };
    let mut tracker = interfaces.progress(format!("Scanned more {} documents", collection));
    for item in cursor {
        if let Err(error) = item {
            let error = format_fail(&error);
            outcomes.error(Error::GenericError(error));
        }
        tracker.track();
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
    info!(logger, "Checking view store schema");

    let config = args.value_of("config").unwrap();
    let config = Config::from_file(config).with_context(|_| ErrorKind::ConfigLoad)?;
    let store = Admin::make(config.storage.view, logger.clone())
        .with_context(|_| ErrorKind::AdminInit("view store"))?;
    let mut outcomes = Outcomes::new();

    debug!(logger, "Checking schema");
    match store.validate().schema() {
        Ok(results) => consume_results(results, &mut outcomes),
        Err(error) => {
            let error = error.context(ErrorKind::CheckFailed("current schema"));
            outcomes.error(Error::GenericError(format_fail(&error)));
        }
    };
    outcomes.report(&logger);

    debug!(logger, "Checking indexes");
    match store.validate().indexes() {
        Ok(results) => consume_results(results, &mut outcomes),
        Err(error) => {
            let error = error.context(ErrorKind::CheckFailed("existing indexes"));
            outcomes.error(Error::GenericError(format_fail(&error)));
        }
    };
    outcomes.report(&logger);

    debug!(logger, "Checking for removed collections/tables or indexes");
    match store.validate().removed_entities() {
        Ok(results) => consume_results(results, &mut outcomes),
        Err(error) => {
            let error = error.context(ErrorKind::CheckFailed("removed collections or indexes"));
            outcomes.error(Error::GenericError(format_fail(&error)));
        }
    };
    outcomes.report(&logger);

    // Finish up.
    if outcomes.has_errors() {
        error!(logger, "View store schema checks failed");
        return Err(ErrorKind::CheckWithErrors("primary store schema").into());
    }
    if outcomes.has_warnings() {
        warn!(logger, "View store schema checks passed with warnings");
        return Ok(());
    }
    info!(logger, "View store schema checks passed");
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

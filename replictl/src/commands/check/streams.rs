use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use error_chain::ChainedError;
use prometheus::Registry;

use replicante::Config;
use replicante_data_store::Store;

use replicante_streams_events::EventsStream;
use replicante_streams_events::ScanFilters;
use replicante_streams_events::ScanOptions;

use super::super::super::Interfaces;
use super::super::super::Result;
use super::super::super::ResultExt;

use super::super::super::outcome::Error;
use super::super::super::outcome::Outcomes;


pub const COMMAND: &str = "streams";
const COMMAND_EVENTS: &str = "events";


/// Configure the `replictl check streams` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Check all streams for incompatibilities")
        .subcommand(
            SubCommand::with_name(COMMAND_EVENTS)
            .about("Check the primary store schema compatibility with this version of replicante")
        )
}


/// Check ALL events in the stream for compatibility with this version of replicante.
///
/// The following checks are performed:
///
///   * Each content item is loaded and parsed.
pub fn events<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let logger = interfaces.logger();
    info!(logger, "Checking events stream");
    let confirm = interfaces.prompt().confirm_danger(
        "About to scan ALL events in the stream. \
        This could impact your production system. \
        Would you like to proceed?"
    )?;
    if !confirm {
        error!(logger, "Cannot check without user confirmation");
        return Err("Operation aborded by the user".into());
    }

    let mut outcomes = Outcomes::new();
    let config = args.value_of("config").unwrap();
    let config = Config::from_file(config).chain_err(|| "Failed to check events")?;
    let registry = Registry::new();
    let store = Store::new(config.storage, logger.clone(), &registry)?;
    let stream = EventsStream::new(config.events.stream, logger.clone(), store);

    info!(logger, "Checking events stream ...");
    let cursor = stream.scan(ScanFilters::all(), ScanOptions::default())
        .chain_err(|| "Failed to check events")?;
    let mut tracker = interfaces.progress("Processed more events");
    for event in cursor {
        if let Err(error) = event {
            let error = error.display_chain().to_string();
            outcomes.error(Error::GenericError(error));
        }
        tracker.track();
    }
    outcomes.report(&logger);

    // Report results.
    if outcomes.has_errors() {
        error!(logger, "Events stream checks failed");
        return Err("Events stream checks failed".into());
    }
    if outcomes.has_warnings() {
        warn!(logger, "Events stream checks passed with warnings");
        return Ok(());
    }
    info!(logger, "Events stream checks passed");
    Ok(())
}


/// Check all streams for incompatibilities.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name();
    match command {
        Some(COMMAND_EVENTS) => events(args, interfaces),
        None => Err("Need a streams check to run".into()),
        _ => Err("Received unrecognised command".into()),
    }
}

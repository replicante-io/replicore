use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;
use slog::error;
use slog::info;
use slog::warn;

use replicante::Config;
use replicante_service_healthcheck::HealthChecks;
use replicante_stream_events::Stream as EventsStream;
use replicante_util_failure::format_fail;
use replicante_util_rndid::RndId;

use crate::outcome::Error;
use crate::outcome::Outcomes;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub const COMMAND: &str = "streams";
const COMMAND_EVENTS: &str = "events";

/// Configure the `replictl check streams` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Check all streams for incompatibilities")
        .subcommand(
            SubCommand::with_name(COMMAND_EVENTS)
                .about("Check all events data for format incompatibilities"),
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
         Would you like to proceed?",
    )?;
    if !confirm {
        error!(logger, "Cannot check without user confirmation");
        return Err(ErrorKind::UserAbort.into());
    }

    let mut healthchecks = HealthChecks::new();
    let mut outcomes = Outcomes::new();
    let config = args.value_of("config").unwrap();
    let config = Config::from_file(config).with_context(|_| ErrorKind::ConfigLoad)?;
    let stream = EventsStream::new(
        config.events.stream,
        logger.clone(),
        &mut healthchecks,
        None,
    )
    .with_context(|_| ErrorKind::ClientInit("events stream"))?;

    info!(logger, "Checking events stream ...");
    warn!(
        logger,
        "This check may block until at least one message is on the stream",
    );
    let group = format!("replictl:events:{}", RndId::new());
    let iter = stream
        .short_follow(group, None)
        .with_context(|_| ErrorKind::CheckFailed("events"))?;
    let mut tracker = interfaces.progress("Processed more events");
    for message in iter {
        match message {
            Err(error) => {
                let error = format_fail(&error);
                outcomes.error(Error::GenericError(error));
            }
            Ok(message) => {
                if let Err(error) = message.payload() {
                    let error = format_fail(&error);
                    outcomes.error(Error::GenericError(error));
                }
                // Ignore errors sending acks for the scan.
                let _ = message.async_ack();
            }
        };
        tracker.track();
    }
    outcomes.report(&logger);

    // Report results.
    if outcomes.has_errors() {
        error!(logger, "Events stream checks failed");
        return Err(ErrorKind::CheckWithErrors("events stream").into());
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
        None => Err(ErrorKind::NoCommand("replictl check streams").into()),
        Some(name) => {
            Err(ErrorKind::UnkownSubcommand("replictl check streams", name.to_string()).into())
        }
    }
}

use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;
use slog::info;
use slog::warn;

use replicante_service_healthcheck::HealthChecks;
use replicante_stream_events::Stream as EventsStream;
use replicante_util_failure::format_fail;
use replicante_util_rndid::RndId;

pub const COMMAND: &str = "events-stream";

use crate::outcome::Error;
use crate::outcome::Outcomes;
use crate::utils::load_config;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND).about("Validate messages on the events stream")
}

pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<Outcomes> {
    let logger = interfaces.logger();
    info!(logger, "Checking events stream");

    let config = load_config(args)?;
    let mut healthchecks = HealthChecks::new();
    let stream = EventsStream::new(config.events, logger.clone(), &mut healthchecks, None)
        .with_context(|_| ErrorKind::ClientInit("events stream"))?;

    warn!(
        logger,
        "This check may block until at least one event is on the stream",
    );
    let group = format!("repliadm:events:{}", RndId::new());
    let iter = stream
        .short_follow(group, None)
        .with_context(|_| ErrorKind::ValidationError("events stream"))?;
    let mut outcomes = Outcomes::new();
    let mut tracker = interfaces.progress("Processed more events");
    for message in iter {
        match message {
            Err(error) => {
                let error = format_fail(&error);
                outcomes.error(Error::Generic(error));
            }
            Ok(message) => {
                if let Err(error) = message.payload() {
                    let error = format_fail(&error);
                    outcomes.error(Error::Generic(error));
                }
                // Ignore errors sending acks for the scan.
                let _ = message.async_ack();
            }
        };
        tracker.track();
    }

    Ok(outcomes)
}

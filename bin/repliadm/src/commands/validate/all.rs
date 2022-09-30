use clap::ArgMatches;
use clap::Command;

pub const COMMAND: &str = "all";

use crate::Interfaces;
use crate::Result;

use crate::outcome::Outcomes;

pub fn command() -> Command {
    Command::new(COMMAND).about("Run all validate operations")
}

pub fn run(args: &ArgMatches, interfaces: &Interfaces) -> Result<Outcomes> {
    let mut outcomes = Outcomes::new();
    let logger = interfaces.logger();

    // Call other validators and report intermediate progress.
    outcomes.extend(super::config::run(args, interfaces)?);
    outcomes.report(logger);

    outcomes.extend(super::coordinator_elections::run(args, interfaces)?);
    outcomes.report(logger);
    outcomes.extend(super::coordinator_nblocks::run(args, interfaces)?);
    outcomes.report(logger);
    outcomes.extend(super::coordinator_nodes::run(args, interfaces)?);
    outcomes.report(logger);

    outcomes.extend(super::events_stream::run(args, interfaces)?);
    outcomes.report(logger);

    outcomes.extend(super::task_queues::run(args, interfaces)?);
    outcomes.report(logger);

    outcomes.extend(super::primary_store_schema::run(args, interfaces)?);
    outcomes.report(logger);
    outcomes.extend(super::primary_store_data::run(args, interfaces)?);
    outcomes.report(logger);
    outcomes.extend(super::view_store_schema::run(args, interfaces)?);
    outcomes.report(logger);
    outcomes.extend(super::view_store_data::run(args, interfaces)?);
    outcomes.report(logger);

    // Report back.
    Ok(outcomes)
}

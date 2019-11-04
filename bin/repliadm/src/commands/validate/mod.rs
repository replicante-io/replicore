use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use slog::error;
use slog::info;
use slog::warn;

pub const COMMAND: &str = "validate";

use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

mod all;
mod config;
mod coordinator_elections;
mod coordinator_nblocks;
mod coordinator_nodes;
mod events_stream;
mod primary_store_data;
mod primary_store_schema;
mod task_queues;
mod update_compatibility;
mod view_store_data;
mod view_store_schema;

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Validate the state of system dependencies")
        .subcommand(all::command())
        .subcommand(config::command())
        .subcommand(coordinator_elections::command())
        .subcommand(coordinator_nblocks::command())
        .subcommand(coordinator_nodes::command())
        .subcommand(events_stream::command())
        .subcommand(primary_store_data::command())
        .subcommand(primary_store_schema::command())
        .subcommand(task_queues::command())
        .subcommand(update_compatibility::command())
        .subcommand(view_store_data::command())
        .subcommand(view_store_schema::command())
}

pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name();

    // Validate as requested and process the outcomes of validation.
    let outcomes = match command {
        Some(all::COMMAND) => all::run(args, interfaces),
        Some(config::COMMAND) => config::run(args, interfaces),
        Some(coordinator_elections::COMMAND) => coordinator_elections::run(args, interfaces),
        Some(coordinator_nblocks::COMMAND) => coordinator_nblocks::run(args, interfaces),
        Some(coordinator_nodes::COMMAND) => coordinator_nodes::run(args, interfaces),
        Some(events_stream::COMMAND) => events_stream::run(args, interfaces),
        Some(primary_store_data::COMMAND) => primary_store_data::run(args, interfaces),
        Some(primary_store_schema::COMMAND) => primary_store_schema::run(args, interfaces),
        Some(task_queues::COMMAND) => task_queues::run(args, interfaces),
        Some(update_compatibility::COMMAND) => update_compatibility::run(args, interfaces),
        Some(view_store_data::COMMAND) => view_store_data::run(args, interfaces),
        Some(view_store_schema::COMMAND) => view_store_schema::run(args, interfaces),
        None => Err(ErrorKind::NoCommand(format!("{} {}", env!("CARGO_PKG_NAME"), COMMAND)).into()),
        Some(name) => Err(ErrorKind::UnkownSubcommand(
            format!("{} {}", env!("CARGO_PKG_NAME"), COMMAND),
            name.to_string(),
        )
        .into()),
    };
    let mut outcomes = outcomes?;

    // Report outcomes once for all validation commands.
    let logger = interfaces.logger();
    outcomes.report(&logger);
    if outcomes.has_errors() {
        error!(logger, "System validation failed");
        return Err(ErrorKind::ValidationFailed.into());
    }
    if outcomes.has_warnings() {
        warn!(logger, "System validation finished with warnings");
        return Ok(());
    }
    info!(logger, "System validation finished without issues");
    Ok(())
}

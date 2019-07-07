use clap::App;
use clap::ArgMatches;
use clap::SubCommand;

mod config;
mod coordinator;
mod stores;
mod streams;
mod tasks;

use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub const COMMAND: &str = "check";
const DEEP_COMMAND: &str = "deep";
const QUICK_COMMAND: &str = "quick";
const UPDATE_COMMAND: &str = "update";

/// Configure the `replictl check` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Perform checks on the system to find issues")
        .subcommand(config::command())
        .subcommand(coordinator::command())
        .subcommand(stores::command())
        .subcommand(streams::command())
        .subcommand(tasks::command())
        .subcommand(
            SubCommand::with_name(DEEP_COMMAND)
                .about("Run all checks INCLUDING the ones that iterate over ALL data"),
        )
        .subcommand(
            SubCommand::with_name(QUICK_COMMAND)
                .about("Run all checks that do NOT iterate over data (default command)"),
        )
        .subcommand(
            SubCommand::with_name(UPDATE_COMMAND)
                .about("Run all checks to confirm an update is possible (iterates over ALL data!)"),
        )
}

/// Switch the control flow to the requested check command.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name();
    match command {
        Some(config::COMMAND) => config::run(args, interfaces),
        Some(coordinator::COMMAND) => coordinator::run(args, interfaces),
        Some(stores::COMMAND) => stores::run(args, interfaces),
        Some(streams::COMMAND) => streams::run(args, interfaces),
        Some(tasks::COMMAND) => tasks::run(args, interfaces),
        Some(DEEP_COMMAND) => run_deep(args, interfaces),
        Some(QUICK_COMMAND) => run_quick(args, interfaces),
        // Currently update is an alias for `deep` but that may change.
        Some(UPDATE_COMMAND) => run_deep(args, interfaces),
        None => run_quick(args, interfaces),
        Some(name) => Err(ErrorKind::UnkownSubcommand("replictl check", name.to_string()).into()),
    }
}

/// Run all checks INCLUDING the ones that iterate over ALL data.
fn run_deep<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let config = config::run(args, interfaces);
    let coordinator = coordinator::run(args, interfaces);
    let stores = stores::run_deep(args, interfaces);
    let streams_events = streams::events(args, interfaces);
    let tasks_data = tasks::data(args, interfaces);
    config?;
    coordinator?;
    stores?;
    streams_events?;
    tasks_data?;
    Ok(())
}

/// Run all checks that do NOT iterate over data.
fn run_quick<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    config::run(args, interfaces)
}

use clap::App;
use clap::ArgMatches;
use clap::SubCommand;

use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

mod election_info;
mod election_list;
mod force_release_nonblocking_lock;
mod nonblocking_lock_info;
mod nonblocking_lock_list;
mod step_down_election;

pub const COMMAND: &str = "coordinator";

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Inspect and manage the coordination service")
        .subcommand(election_info::command())
        .subcommand(election_list::command())
        .subcommand(force_release_nonblocking_lock::command())
        .subcommand(nonblocking_lock_info::command())
        .subcommand(nonblocking_lock_list::command())
        .subcommand(step_down_election::command())
}

pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name();

    match command {
        Some(election_info::COMMAND) => election_info::run(args, interfaces),
        Some(election_list::COMMAND) => election_list::run(args, interfaces),
        Some(force_release_nonblocking_lock::COMMAND) => force_release_nonblocking_lock::run(args, interfaces),
        Some(nonblocking_lock_info::COMMAND) => nonblocking_lock_info::run(args, interfaces),
        Some(nonblocking_lock_list::COMMAND) => nonblocking_lock_list::run(args, interfaces),
        Some(step_down_election::COMMAND) => step_down_election::run(args, interfaces),
        None => Err(ErrorKind::NoCommand(format!("{} {}", env!("CARGO_PKG_NAME"), COMMAND)).into()),
        Some(name) => Err(ErrorKind::UnkownSubcommand(
            format!("{} {}", env!("CARGO_PKG_NAME"), COMMAND),
            name.to_string(),
        )
        .into()),
    }
}

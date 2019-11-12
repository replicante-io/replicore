use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use slog::Logger;

use crate::ErrorKind;
use crate::Result;
use crate::CLI_NAME;

mod list_sessions;
mod login;
mod logout;
mod session_info;
mod set_default_session;

pub const COMMAND: &str = "sso";

pub fn command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND)
        .about("Login, logout, manage authentication and sessions")
        .subcommand(list_sessions::command())
        .subcommand(login::command())
        .subcommand(logout::command())
        .subcommand(session_info::command())
        .subcommand(set_default_session::command())
}

pub fn run<'a>(cli: &ArgMatches<'a>, logger: &Logger) -> Result<()> {
    let command = cli.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name();

    match command {
        Some(list_sessions::COMMAND) => list_sessions::run(cli, logger),
        Some(login::COMMAND) => login::run(cli, logger),
        Some(logout::COMMAND) => logout::run(cli, logger),
        Some(session_info::COMMAND) => session_info::run(cli, logger),
        Some(set_default_session::COMMAND) => set_default_session::run(cli, logger),
        None => Err(ErrorKind::NoCommand(format!("{} {}", CLI_NAME, COMMAND)).into()),
        Some(name) => Err(ErrorKind::UnkownSubcommand(
            format!("{} {}", CLI_NAME, COMMAND),
            name.to_string(),
        )
        .into()),
    }
}

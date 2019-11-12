use clap::App;
use clap::ArgMatches;
use slog::Logger;

use super::CLI_NAME;
use crate::ErrorKind;
use crate::Result;

mod sso;

/// Configure the given `clap::App` with top-level commands.
pub fn configure_cli<'a, 'b>(cli: App<'a, 'b>) -> App<'a, 'b> {
    cli.subcommand(sso::command())
}

/// Execute the selected replictl command.
pub fn execute<'a>(cli: &ArgMatches<'a>, logger: &Logger) -> Result<()> {
    match cli.subcommand_name() {
        Some(sso::COMMAND) => sso::run(cli, logger),
        None => Err(ErrorKind::NoCommand(CLI_NAME.to_string()).into()),
        Some(name) => {
            Err(ErrorKind::UnkownSubcommand(CLI_NAME.to_string(), name.to_string()).into())
        }
    }
}

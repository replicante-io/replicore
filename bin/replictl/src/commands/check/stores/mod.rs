use clap::App;
use clap::ArgMatches;
use clap::SubCommand;

use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

mod primary;

pub const COMMAND: &str = "stores";

/// Configure the `replictl check stores` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Check stores for incompatibilities")
        .subcommand(primary::command())
}

/// Switch the control flow to the requested check command.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name();
    match command {
        Some(primary::COMMAND) => primary::run(args, interfaces),
        None => Err(ErrorKind::NoCommand("replictl check stores").into()),
        Some(name) => {
            Err(ErrorKind::UnkownSubcommand("replictl check stores", name.to_string()).into())
        }
    }
}

/// Run all checks INCLUDING the ones that iterate over ALL data.
pub fn run_deep<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let schema = primary::schema(args, interfaces);
    let data = primary::data(args, interfaces);
    schema?;
    data?;
    Ok(())
}

use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use failure::err_msg;

use super::super::ErrorKind;
use super::super::Interfaces;
use super::super::Result;


mod nblock;


pub const COMMAND: &str = "coordinator";


/// Configure the `replictl coordinator` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Inspect and manage cluster coordination")
        .subcommand(nblock::command())
}


/// Switch the control flow to the requested coordinator command.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name();
    match command {
        Some(nblock::COMMAND) => nblock::run(args, interfaces),
        None => Err(ErrorKind::Legacy(err_msg("need a coordinator command to run")).into()),
        _ => Err(ErrorKind::Legacy(err_msg("received unrecognised command")).into()),
    }
}

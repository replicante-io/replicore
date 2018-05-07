use clap::App;
use clap::ArgMatches;
use clap::SubCommand;

use super::super::Result;


mod config;


pub const COMMAND: &'static str = "check";


/// Configure the `replictl check` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("TODO")
        .subcommand(config::command())
}


/// Switch the control flow to the requested check command.
pub fn run<'a>(args: ArgMatches<'a>) -> Result<()> {
    match args.subcommand_matches(COMMAND).unwrap().subcommand_name() {
        Some(config::COMMAND) => config::run(args),
        None => Err("TODO: run all".into()),
        _ => Err("Received unrecognised command".into()),
    }
}

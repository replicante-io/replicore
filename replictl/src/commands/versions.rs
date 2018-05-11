use clap::App;
use clap::ArgMatches;
use clap::SubCommand;


use super::super::Interfaces;
use super::super::Result;


pub const COMMAND: &'static str = "versions";


/// Configure the `replictl check` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Reports version information for various systems")
}


/// Switch the control flow to the requested check command.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    // TODO
    Ok(())
}

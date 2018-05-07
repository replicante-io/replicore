use clap::App;
use clap::ArgMatches;
use clap::SubCommand;

use super::super::super::Result;

pub const COMMAND: &'static str = "config";


/// TODO
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("TODO")
}


/// TODO
pub fn run<'a>(_args: ArgMatches<'a>) -> Result<()> {
    println!("TODO: implement check config");
    Ok(())
}


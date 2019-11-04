use clap::App;
use clap::ArgMatches;
use clap::SubCommand;

pub const COMMAND: &str = "update-compatibility";

use crate::outcome::Outcomes;
use crate::Interfaces;
use crate::Result;

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Validate a running system against a new version of replicante")
}

// For now this is an alias to `all` but this may change in the future.
// The name also makes the purpose much clearer.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<Outcomes> {
    super::all::run(args, interfaces)
}

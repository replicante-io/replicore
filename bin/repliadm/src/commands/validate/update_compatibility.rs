
use clap::ArgMatches;
use clap::Command;

pub const COMMAND: &str = "update-compatibility";

use crate::outcome::Outcomes;
use crate::Interfaces;
use crate::Result;

pub fn command() -> Command {
    Command::new(COMMAND)
        .about("Validate a running system against a new version of replicante")
}

// For now this is an alias to `all` but this may change in the future.
// The name also makes the purpose much clearer.
pub fn run(args: &ArgMatches, interfaces: &Interfaces) -> Result<Outcomes> {
    super::all::run(args, interfaces)
}

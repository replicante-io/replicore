use clap::ArgMatches;
use clap::Command;
use failure::ResultExt;

pub const COMMAND: &str = "election-list";

use crate::utils::coordinator_admin;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub fn command() -> Command {
    Command::new(COMMAND).about("List all registered elections")
}

pub fn run(args: &ArgMatches, interfaces: &Interfaces) -> Result<()> {
    let logger = interfaces.logger();
    let admin = coordinator_admin(args, logger.clone())?;
    println!("==> Available elections:");
    for election in admin.elections() {
        let election = election.with_context(|_| ErrorKind::CoordinatorElectionList)?;
        println!("====> {}", election.name());
    }
    Ok(())
}

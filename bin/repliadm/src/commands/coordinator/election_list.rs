use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;

pub const COMMAND: &str = "election-list";

use crate::utils::coordinator_admin;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("List all registered elections")
}

pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let logger = interfaces.logger();
    let admin = coordinator_admin(args, logger.clone())?;
    println!("==> Available elections:");
    for election in admin.elections() {
        let election = election.with_context(|_| ErrorKind::CoordinatorElectionList)?;
        println!("====> {}", election.name());
    }
    Ok(())
}

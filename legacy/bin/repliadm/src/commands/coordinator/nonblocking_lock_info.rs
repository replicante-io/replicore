use clap::Arg;
use clap::ArgMatches;
use clap::Command;
use failure::ResultExt;

pub const COMMAND: &str = "nonblocking-lock-info";

use crate::utils::coordinator_admin;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub fn command() -> Command {
    Command::new(COMMAND)
        .about("Show information about a non-blocking lock")
        .arg(
            Arg::new("lock")
                .long("lock")
                .help("Name of the lock to lookup")
                .value_name("LOCK")
                .num_args(1)
                .required(true),
        )
}

pub fn run(args: &ArgMatches, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let name = command.get_one::<String>("lock").unwrap();

    let logger = interfaces.logger();
    let admin = coordinator_admin(args, logger.clone())?;
    let lock = admin
        .non_blocking_lock(name)
        .with_context(|_| ErrorKind::CoordinatorNBLockLookup(name.to_string()))?;
    let owner = lock
        .owner()
        .with_context(|_| ErrorKind::CoordinatorNBLockOwnerLookup(name.to_string()))?;
    println!("==> Lock name: {}", lock.name());
    println!("==> Node ID currently holding the lock: {}", owner);

    Ok(())
}

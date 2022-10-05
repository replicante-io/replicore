use clap::ArgMatches;
use clap::Command;
use failure::ResultExt;

pub const COMMAND: &str = "nonblocking-lock-list";

use crate::utils::coordinator_admin;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub fn command() -> Command {
    Command::new(COMMAND).about("List currently held non-blocking locks")
}

pub fn run(args: &ArgMatches, interfaces: &Interfaces) -> Result<()> {
    let logger = interfaces.logger();
    let admin = coordinator_admin(args, logger.clone())?;
    println!("==> Currently held locks:");
    for lock in admin.non_blocking_locks() {
        let lock = lock.with_context(|_| ErrorKind::CoordinatorNBLockList)?;
        println!("====> {}", lock.name());
    }
    Ok(())
}

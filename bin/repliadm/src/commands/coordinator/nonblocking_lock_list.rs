use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;

pub const COMMAND: &str = "nonblocking-lock-list";

use crate::utils::coordinator_admin;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("List currently held non-blocking locks")
}

pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let logger = interfaces.logger();
    let admin = coordinator_admin(args, logger.clone())?;
    println!("==> Currently held locks:");
    for lock in admin.non_blocking_locks() {
        let lock = lock.with_context(|_| ErrorKind::CoordinatorNBLockList)?;
        println!("====> {}", lock.name());
    }
    Ok(())
}

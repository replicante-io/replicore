use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;

pub const COMMAND: &str = "force-release-nonblocking-lock";

use crate::utils::coordinator_admin;
use crate::utils::take_responsibility_arg;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("*** DANGER *** Force a held lock to be released")
        .arg(
            Arg::with_name("lock")
                .long("lock")
                .help("Name of the lock to release")
                .value_name("LOCK")
                .takes_value(true)
                .required(true),
        )
        .arg(take_responsibility_arg())
}

pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let name = command.value_of("lock").unwrap();

    println!("==> *** DANGER ***");
    println!("==> You should not be force-releasing locks without intimate knowledge of the code");
    println!(
        "==> Only do this if the process holding the lock did not release it after a kill -9 \
         (give it a minute for the coordinator to detect the process as dead)"
    );
    println!("==> If you do need to force-release don't forget to also report it as a bug:");
    println!("==>   https://github.com/replicante-io/replicante/issues");
    println!("==> *** DANGER ***");
    if !command.is_present("take-responsibility") {
        return Err(ErrorKind::TakeResponsibility.into());
    }

    let logger = interfaces.logger();
    let admin = coordinator_admin(args, logger.clone())?;
    let mut lock = admin
        .non_blocking_lock(name)
        .with_context(|_| ErrorKind::CoordinatorNBLockLookup(name.to_string()))?;
    lock.force_release()
        .with_context(|_| ErrorKind::CoordinatorNBLockRelease(name.to_string()))?;
    println!("==> Lock released by force");

    Ok(())
}

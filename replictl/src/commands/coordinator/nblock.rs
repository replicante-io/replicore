use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;

use super::super::super::ErrorKind;
use super::super::super::Interfaces;
use super::super::super::Result;
use super::admin_interface;


pub const COMMAND: &str = "nb-lock";
const COMMAND_FORCE_RELEASE: &str = "force-release";
const COMMAND_INFO: &str = "info";
const COMMAND_LS: &str = "ls";


/// Configure the `replictl coordinator nb-lock` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Inspect and manage distributed non-blocking locks")
        .subcommand(
            SubCommand::with_name(COMMAND_FORCE_RELEASE)
            .about("*** DANGER *** Force a held lock to be released")
            .arg(
                Arg::with_name("LOCK")
                .help("Name of the lock to release")
                .required(true)
                .index(1)
            )
        )
        .subcommand(
            SubCommand::with_name(COMMAND_INFO)
            .about("Show information about a non-blocking lock")
            .arg(
                Arg::with_name("LOCK")
                .help("Name of the lock to lookup")
                .required(true)
                .index(1)
            )
        )
        .subcommand(
            SubCommand::with_name(COMMAND_LS)
            .about("List currently held non-blocking locks")
        )
}


/// *** DANGER *** Force a held lock to be released.
fn force_release<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND_FORCE_RELEASE).unwrap();
    let name = command.value_of("LOCK").unwrap();
    println!("==> *** DANGER ***");
    println!("==> You should not be force-releasing locks without intimate knowledge of the code");
    println!(
        "==> Only do this if the process holding the lock did not release it after a kill -9 \
        (give it a minute for the coordinator to detect the process as dead)"
    );
    println!("==> If you do need to force-release don't forget to also report it as a bug:");
    println!("==>   https://github.com/replicante-io/replicante/issues");
    println!("==> *** DANGER ***");

    let confirm = interfaces.prompt().confirm_danger(
        &format!("Are you sure you want to force-release non-blocking lock '{}'?", name)
    )?;
    if !confirm {
        println!("==> Not force-releasing the lock");
        return Ok(())
    }
    let admin = admin_interface(args, interfaces)?;
    let mut lock = admin.non_blocking_lock(&name)
        .with_context(|_| ErrorKind::CoordinatorNBLockLookup(name.to_string()))?;
    lock.force_release().with_context(|_| ErrorKind::CoordinatorNBLockRelease(name.to_string()))?;
    println!("==> Lock released by force");
    Ok(())
}


/// Show information about a non-blocking lock.
fn info<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND_INFO).unwrap();
    let name = command.value_of("LOCK").unwrap();
    let admin = admin_interface(args, interfaces)?;
    let lock = admin.non_blocking_lock(&name)
        .with_context(|_| ErrorKind::CoordinatorNBLockLookup(name.to_string()))?;
    let owner = lock.owner()
        .with_context(|_| ErrorKind::CoordinatorNBLockOwnerLookup(name.to_string()))?;
    println!("==> Lock name: {}", lock.name());
    println!("==> Node ID currently holding the lock: {}", owner);
    Ok(())
}


/// List currently held non-blocking locks.
fn ls<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let admin = admin_interface(args, interfaces)?;
    println!("==> Currently held locks:");
    for lock in admin.non_blocking_locks() {
        let lock = lock.with_context(|_| ErrorKind::CoordinatorNBLockList)?;
        println!("====> {}", lock.name());
    }
    Ok(())
}


/// Switch the control flow to the requested non-blocking lock command.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name();
    match command {
        Some(COMMAND_FORCE_RELEASE) => force_release(args, interfaces),
        Some(COMMAND_INFO) => info(args, interfaces),
        Some(COMMAND_LS) => ls(args, interfaces),
        None => Err(ErrorKind::NoCommand("replictl coordinator nb-lock").into()),
        Some(name) => Err(
            ErrorKind::UnkownSubcommand("replictl coordinator nb-lock", name.to_string()).into()
        )
    }
}

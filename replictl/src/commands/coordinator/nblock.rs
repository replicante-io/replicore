use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;
use failure::err_msg;

use replicante::Config;
use replicante_coordinator::Admin;

use super::super::super::ErrorKind;
use super::super::super::Interfaces;
use super::super::super::Result;


pub const COMMAND: &str = "nb-lock";
const COMMAND_INFO: &str = "info";
const COMMAND_LS: &str = "ls";


/// Configure the `replictl coordinator` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Inspect and manage distributed non-blocking locks")
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


/// Helper function to configure and instantiate an Admin interface.
fn admin_interface<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<Admin> {
    let logger = interfaces.logger().clone();
    let config = args.value_of("config").unwrap();
    let config = Config::from_file(config)
        .context(ErrorKind::Legacy(err_msg("failed to initialise coordinator interface")))?;
    let admin = Admin::new(config.coordinator, logger)
        .context(ErrorKind::Legacy(err_msg("failed to initialise coordinator interface")))?;
    Ok(admin)
}


/// Show information about a non-blocking lock.
fn info<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND_INFO).unwrap();
    let name = command.value_of("LOCK").unwrap();
    let admin = admin_interface(args, interfaces)?;
    let lock = admin.non_blocking_lock(&name)
        .context(ErrorKind::Legacy(err_msg("failed to lookup node")))?;
    let owner = lock.owner()
        .context(ErrorKind::Legacy(err_msg("lock owner lookup failed")))?;
    println!("==> Lock name: {}", lock.name());
    println!("==> Node ID currently holding the lock: {}", owner);
    Ok(())
}


/// List currently held non-blocking locks.
fn ls<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let admin = admin_interface(args, interfaces)?;
    println!("==> Currently held locks:");
    for lock in admin.non_blocking_locks() {
        let lock = lock.context(ErrorKind::Legacy(err_msg("failed to list non-blocking locks")))?;
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
        Some(COMMAND_INFO) => info(args, interfaces),
        Some(COMMAND_LS) => ls(args, interfaces),
        None => Err(ErrorKind::Legacy(err_msg("need a coordinator nb-lock command to run")).into()),
        _ => Err(ErrorKind::Legacy(err_msg("received unrecognised command")).into()),
    }
}

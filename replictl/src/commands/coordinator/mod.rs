use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;
use failure::err_msg;

use replicante::Config;
use replicante_coordinator::Admin;

use super::super::ErrorKind;
use super::super::Interfaces;
use super::super::Result;


mod election;
mod nblock;


pub const COMMAND: &str = "coordinator";


/// Configure the `replictl coordinator` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Inspect and manage cluster coordination")
        .subcommand(election::command())
        .subcommand(nblock::command())
}


/// Switch the control flow to the requested coordinator command.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name();
    match command {
        Some(election::COMMAND) => election::run(args, interfaces),
        Some(nblock::COMMAND) => nblock::run(args, interfaces),
        None => Err(ErrorKind::Legacy(err_msg("need a coordinator command to run")).into()),
        _ => Err(ErrorKind::Legacy(err_msg("received unrecognised command")).into()),
    }
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

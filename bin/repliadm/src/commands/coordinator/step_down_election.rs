use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;
use slog::info;

pub const COMMAND: &str = "step-down-election";

use crate::utils::coordinator_admin;
use crate::utils::take_responsibility_arg;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("*** DANGER *** Force a primary to be re-elected")
        .arg(
            Arg::with_name("election")
                .long("election")
                .help("Name of the election to step-down")
                .value_name("ELECTION")
                .takes_value(true)
                .required(true),
        )
        .arg(take_responsibility_arg())
}

pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let name = command.value_of("election").unwrap();
    if !command.is_present("take-responsibility") {
        return Err(ErrorKind::TakeResponsibility.into());
    }

    let logger = interfaces.logger();
    let admin = coordinator_admin(args, logger.clone())?;
    let election = admin
        .election(&name)
        .with_context(|_| ErrorKind::CoordinatorElectionLookup(name.to_string()))?;
    let stepped_down = election
        .step_down()
        .with_context(|_| ErrorKind::CoordinatorElectionStepDown(name.to_string()))?;
    if stepped_down {
        info!(logger, "Stepped down election"; "election" => name);
    } else {
        info!(logger, "No need to step down election"; "election" => name);
    }

    Ok(())
}

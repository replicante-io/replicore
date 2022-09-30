use clap::Arg;
use clap::ArgMatches;
use clap::Command;
use failure::ResultExt;
use slog::info;

pub const COMMAND: &str = "step-down-election";

use crate::utils::coordinator_admin;
use crate::utils::take_responsibility_arg;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub fn command() -> Command {
    Command::new(COMMAND)
        .about("*** DANGER *** Force a primary to be re-elected")
        .arg(
            Arg::new("election")
                .long("election")
                .help("Name of the election to step-down")
                .value_name("ELECTION")
                .num_args(1)
                .required(true),
        )
        .arg(take_responsibility_arg())
}

pub fn run(args: &ArgMatches, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let name = command.get_one::<String>("election").unwrap();
    if !command.get_flag("take-responsibility") {
        return Err(ErrorKind::TakeResponsibility.into());
    }

    let logger = interfaces.logger();
    let admin = coordinator_admin(args, logger.clone())?;
    let election = admin
        .election(name)
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

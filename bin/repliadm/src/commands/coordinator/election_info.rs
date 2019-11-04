use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;

pub const COMMAND: &str = "election-info";

use crate::utils::coordinator_admin;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Show information about an election")
        .arg(
            Arg::with_name("election")
                .long("election")
                .help("Name of the election to lookup")
                .value_name("ELECTION")
                .takes_value(true)
                .required(true),
        )
}

pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let name = command.value_of("election").unwrap();

    let logger = interfaces.logger();
    let admin = coordinator_admin(args, logger.clone())?;
    let election = admin
        .election(&name)
        .with_context(|_| ErrorKind::CoordinatorElectionLookup(name.to_string()))?;
    println!("==> Election name: {}", election.name());

    let primary = election
        .primary()
        .with_context(|_| ErrorKind::CoordinatorElectionPrimaryLookup(name.to_string()))?;
    let primary = match primary {
        None => "NONE ELECTED".into(),
        Some(node_id) => node_id.to_string(),
    };
    println!("==> Election primary: {}", primary);

    let secondaries_count = election
        .secondaries_count()
        .with_context(|_| ErrorKind::CoordinatorElectionSecondaryCount(name.to_string()))?;
    println!("==> Election secondaries count: {}", secondaries_count);

    Ok(())
}

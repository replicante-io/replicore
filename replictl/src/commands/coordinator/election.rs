use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;
use slog::info;

use super::super::super::ErrorKind;
use super::super::super::Interfaces;
use super::super::super::Result;
use super::admin_interface;

pub const COMMAND: &str = "election";
const COMMAND_INFO: &str = "info";
const COMMAND_LS: &str = "ls";
const COMMAND_STEP_DOWN: &str = "step-down";

/// Configure the `replictl coordinator election` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Inspect and manage distributed elections")
        .subcommand(
            SubCommand::with_name(COMMAND_INFO)
                .about("Show information about an election")
                .arg(
                    Arg::with_name("ELECTION")
                        .help("Name of the election to lookup")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(SubCommand::with_name(COMMAND_LS).about("List all elections"))
        .subcommand(
            SubCommand::with_name(COMMAND_STEP_DOWN)
                .about("Strip the current primary of its role and forces a new election")
                .arg(
                    Arg::with_name("ELECTION")
                        .help("Name of the election to step-down")
                        .required(true)
                        .index(1),
                ),
        )
}

/// Show information about a non-blocking lock.
fn info<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND_INFO).unwrap();
    let name = command.value_of("ELECTION").unwrap();
    let admin = admin_interface(args, interfaces)?;
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

/// List available elections.
fn ls<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let admin = admin_interface(args, interfaces)?;
    println!("==> Available elections:");
    for election in admin.elections() {
        let election = election.with_context(|_| ErrorKind::CoordinatorElectionList)?;
        println!("====> {}", election.name());
    }
    Ok(())
}

/// Strip the current primary of its role and forces a new election.
fn step_down<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND_STEP_DOWN).unwrap();
    let name = command.value_of("ELECTION").unwrap();
    let admin = admin_interface(args, interfaces)?;
    let election = admin
        .election(&name)
        .with_context(|_| ErrorKind::CoordinatorElectionLookup(name.to_string()))?;
    let stepped_down = election
        .step_down()
        .with_context(|_| ErrorKind::CoordinatorElectionStepDown(name.to_string()))?;
    let logger = interfaces.logger();
    if stepped_down {
        info!(logger, "Stepped down election"; "election" => name);
    } else {
        info!(logger, "No need to step down election"; "election" => name);
    }
    Ok(())
}

/// Switch the control flow to the requested election command.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name();
    match command {
        Some(COMMAND_INFO) => info(args, interfaces),
        Some(COMMAND_LS) => ls(args, interfaces),
        Some(COMMAND_STEP_DOWN) => step_down(args, interfaces),
        None => Err(ErrorKind::NoCommand("replictl coordinator election").into()),
        Some(name) => Err(ErrorKind::UnkownSubcommand(
            "replictl coordinator election",
            name.to_string(),
        )
        .into()),
    }
}

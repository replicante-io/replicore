use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;
use failure::err_msg;

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
                .index(1)
            )
        )
        .subcommand(
            SubCommand::with_name(COMMAND_LS)
            .about("List all elections")
        )
        .subcommand(
            SubCommand::with_name(COMMAND_STEP_DOWN)
            .about("Strip the current primary of its role and forces a new election")
            .arg(
                Arg::with_name("ELECTION")
                .help("Name of the election to step-down")
                .required(true)
                .index(1)
            )
        )
}


/// Show information about a non-blocking lock.
fn info<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND_INFO).unwrap();
    let name = command.value_of("ELECTION").unwrap();
    let admin = admin_interface(args, interfaces)?;
    let election = admin.election(&name)
        .context(ErrorKind::Legacy(err_msg("failed to lookup election")))?;
    let primary = election.primary()
        .context(ErrorKind::Legacy(err_msg("election primary lookup failed")))?;
    let primary = match primary {
        None => "NONE ELECTED".into(),
        Some(node_id) => node_id.to_string(),
    };
    let secondaries_count = election.secondaries_count()
        .context(ErrorKind::Legacy(err_msg("election secondaries count failed")))?;
    println!("==> Election name: {}", election.name());
    println!("==> Election primary: {}", primary);
    println!("==> Election secondaries count: {}", secondaries_count);
    Ok(())
}


/// List available elections.
fn ls<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let admin = admin_interface(args, interfaces)?;
    println!("==> Available elections:");
    for election in admin.elections() {
        let election = election.context(ErrorKind::Legacy(err_msg("failed to list elections")))?;
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
    let election = admin.election(&name)
        .context(ErrorKind::Legacy(err_msg("failed to lookup election")))?;
    let stepped_down = election.step_down()
        .context(ErrorKind::Legacy(err_msg("failed to step-down election")))?;
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
        None => Err(ErrorKind::Legacy(err_msg("need a coordinator election command to run")).into()),
        _ => Err(ErrorKind::Legacy(err_msg("received unrecognised command")).into()),
    }
}

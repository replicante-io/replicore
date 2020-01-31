use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use slog::Logger;

use replicante_models_core::api::apply::SCOPE_CLUSTER;
use replicante_models_core::api::apply::SCOPE_NS;

use crate::ErrorKind;
use crate::Result;
use crate::CLI_NAME;

mod approve;
mod disapprove;

pub const COMMAND: &str = "action";
const CLI_ACTION_ID: &str = "action";

pub fn command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND)
        .about("Manage and inspect actions")
        .arg(
            Arg::with_name(SCOPE_CLUSTER)
                .long("cluster")
                .value_name("CLUSTER")
                .takes_value(true)
                .env("REPLICTL_CLUSTER")
                .required(true)
                .help("ID of the cluster the action is for"),
        )
        .arg(
            Arg::with_name(SCOPE_NS)
                .long("namespace")
                .value_name("NAMESPACE")
                .takes_value(true)
                .env("REPLICTL_NS")
                .required(true)
                .help("Namespace that contains the cluster and action"),
        )
        .arg(
            Arg::with_name(CLI_ACTION_ID)
                .long("action")
                .value_name("ACTION")
                .takes_value(true)
                .env("REPLICTL_ACTION")
                .required(true)
                .help("ID of the action to operate on"),
        )
        .subcommand(approve::command())
        .subcommand(disapprove::command())
}

pub fn run<'a>(cli: &ArgMatches<'a>, logger: &Logger) -> Result<()> {
    let command = cli.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name();

    match command {
        Some(approve::COMMAND) => approve::run(cli, logger),
        Some(disapprove::COMMAND) => disapprove::run(cli, logger),
        None => Err(ErrorKind::NoCommand(format!("{} {}", CLI_NAME, COMMAND)).into()),
        Some(name) => Err(ErrorKind::UnkownSubcommand(
            format!("{} {}", CLI_NAME, COMMAND),
            name.to_string(),
        )
        .into()),
    }
}

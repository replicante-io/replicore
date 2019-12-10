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

mod refresh;

pub const COMMAND: &str = "cluster";

pub fn command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND)
        .about("Manage and inspect clusters")
        .arg(
            Arg::with_name(SCOPE_CLUSTER)
                .long("cluster")
                .value_name("CLUSTER")
                .takes_value(true)
                .env("REPLICTL_CLUSTER")
                .required(true)
                .help("ID of the cluster to refresh"),
        )
        .arg(
            Arg::with_name(SCOPE_NS)
                .long("namespace")
                .value_name("NAMESPACE")
                .takes_value(true)
                .env("REPLICTL_NS")
                .required(true)
                .help("Namespace that contains the cluster to refresh"),
        )
        .subcommand(refresh::command())
}

pub fn run<'a>(cli: &ArgMatches<'a>, logger: &Logger) -> Result<()> {
    let command = cli.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name();

    match command {
        Some(refresh::COMMAND) => refresh::run(cli, logger),
        None => Err(ErrorKind::NoCommand(format!("{} {}", CLI_NAME, COMMAND)).into()),
        Some(name) => Err(ErrorKind::UnkownSubcommand(
            format!("{} {}", CLI_NAME, COMMAND),
            name.to_string(),
        )
        .into()),
    }
}

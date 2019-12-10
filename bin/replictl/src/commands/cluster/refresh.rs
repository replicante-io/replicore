use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use slog::Logger;

use replicante_models_core::api::apply::SCOPE_CLUSTER;
use replicante_models_core::api::apply::SCOPE_NS;

use crate::apiclient::RepliClient;
use crate::ErrorKind;
use crate::Result;

pub const COMMAND: &str = "refresh";

pub fn command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND).about("Schedule a cluster refresh task")
}

pub fn run<'a>(cli: &ArgMatches<'a>, logger: &Logger) -> Result<()> {
    // Get cluster selector from CLI args.
    let cli_cluster = cli
        .subcommand_matches(super::COMMAND)
        .expect("must reach this path from `replictl cluster`");
    let _ns = match cli_cluster.value_of(SCOPE_NS) {
        Some(ns) => ns,
        None => {
            let error = ErrorKind::CliOptMissing(SCOPE_NS);
            return Err(error.into());
        }
    };
    let cluster = match cli_cluster.value_of(SCOPE_CLUSTER) {
        Some(cluster) => cluster,
        None => {
            let error = ErrorKind::CliOptMissing(SCOPE_CLUSTER);
            return Err(error.into());
        }
    };

    // Instantiate a client to connect to the API.
    // The session to use is auto-detected.
    let client = RepliClient::from_cli(cli, logger)?;
    client.cluster_refresh(cluster)?;

    println!("Cluster refresh scheduled");
    Ok(())
}

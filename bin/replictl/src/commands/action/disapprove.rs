use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use failure::Fail;
use slog::Logger;
use uuid::Uuid;

use replicante_models_core::api::apply::SCOPE_CLUSTER;
use replicante_models_core::api::apply::SCOPE_NS;

use crate::apiclient::RepliClient;
use crate::ErrorKind;
use crate::Result;

pub const COMMAND: &str = "disapprove";

pub fn command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND).about("Disapprove (reject) an action pending approval")
}

pub fn run<'a>(cli: &ArgMatches<'a>, logger: &Logger) -> Result<()> {
    // Get cluster selector from CLI args.
    let cli_action = cli
        .subcommand_matches(super::COMMAND)
        .expect("must reach this path from `replictl action`");
    let _ns = match cli_action.value_of(SCOPE_NS) {
        Some(ns) => ns,
        None => {
            let error = ErrorKind::CliOptMissing(SCOPE_NS);
            return Err(error.into());
        }
    };
    let cluster = match cli_action.value_of(SCOPE_CLUSTER) {
        Some(cluster) => cluster,
        None => {
            let error = ErrorKind::CliOptMissing(SCOPE_CLUSTER);
            return Err(error.into());
        }
    };
    let action = match cli_action.value_of(super::CLI_ACTION_ID) {
        Some(action) => match Uuid::parse_str(action) {
            Ok(action) => action,
            Err(error) => {
                let error = error.context(ErrorKind::CliOptInvalid(super::CLI_ACTION_ID));
                return Err(error.into());
            }
        },
        None => {
            let error = ErrorKind::CliOptMissing(super::CLI_ACTION_ID);
            return Err(error.into());
        }
    };

    // Instantiate a client to connect to the API.
    // The session to use is auto-detected.
    let client = RepliClient::from_cli(cli, logger)?;
    client.action_disapprove(cluster, action)?;

    println!("Action disapproved and will not be scheduled");
    Ok(())
}

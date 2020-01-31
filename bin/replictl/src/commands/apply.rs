use std::fs::File;

use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;
use serde_json::Value;
use slog::debug;
use slog::info;
use slog::Logger;

use replicante_models_core::api::apply::ApplyObject;
use replicante_models_core::api::apply::SCOPE_ATTRS;
use replicante_models_core::api::apply::SCOPE_CLUSTER;
use replicante_models_core::api::apply::SCOPE_NODE;
use replicante_models_core::api::apply::SCOPE_NS;

use crate::apiclient::RepliClient;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

pub const COMMAND: &str = "apply";

pub fn command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND).about(
        "Apply changes as decribed by the YAML input (heavily inspired by https://kubernetes.io/)",
    )
    .arg(
        Arg::with_name("file")
            .short("f")
            .long("file")
            .value_name("FILE")
            .takes_value(true)
            .required(true)
            .help("Path to a YAML file to apply or - to read from stdin"),
    )
    .arg(
        Arg::with_name(SCOPE_CLUSTER)
            .long("cluster")
            .value_name("CLUSTER")
            .takes_value(true)
            .env("REPLICTL_CLUSTER")
            .help("Override or set the value of metadata.cluster"),
    )
    .arg(
        Arg::with_name(SCOPE_NODE)
            .long("node")
            .value_name("NODE")
            .takes_value(true)
            .env("REPLICTL_NODE")
            .help("Override or set the value of metadata.node"),
    )
    .arg(
        Arg::with_name(SCOPE_NS)
            .long("namespace")
            .value_name("NAMESPACE")
            .takes_value(true)
            .env("REPLICTL_NS")
            .help("Override or set the value of metadata.namespace"),
    )
}

pub fn run<'a>(cli: &ArgMatches<'a>, logger: &Logger) -> Result<()> {
    // Load and validate the object to apply.
    let object = from_yaml(cli, logger)?;
    let mut object = ApplyObject::from_raw(object).map_err(ErrorKind::ApplyValidation)?;

    // Apply scope overrides as needed.
    let cli_apply = cli
        .subcommand_matches(COMMAND)
        .expect("must reach this path from `replictl apply`");
    for scope in SCOPE_ATTRS {
        if let Some(value) = cli_apply.value_of(scope) {
            debug!(
                logger,
                "Overriding scope value from CLI arguments";
                "scope" => scope,
                "value" => value,
            );
            object.metadata.insert(str::to_string(*scope), value.into());
        }
    }

    // Instantiate a client to connect to the API.
    // The session to use is auto-detected.
    let client = RepliClient::from_cli(cli, logger)?;
    let response = match client.apply(object) {
        Ok(response) => response,
        Err(error) => {
            eprintln!("Unable to apply object due to API error");
            return Err(error);
        }
    };
    if let Some(message) = response.get("message") {
        println!("[remote] {}", message);
    }
    println!("Object applied successfully");
    Ok(())
}

/// YAML-decode the apply object from `FILE` or stdin.
fn from_yaml<'a>(cli: &ArgMatches<'a>, logger: &Logger) -> Result<Value> {
    let file = cli
        .subcommand_matches(COMMAND)
        .expect("must reach this path from `replictl apply`")
        .value_of("file")
        .expect("--file should be a required argument in clap");
    info!(logger, "Loading apply data"; "format" => "yaml", "file" => file);
    if file == "-" {
        let stdin = std::io::stdin();
        serde_yaml::from_reader(stdin)
            .with_context(|_| ErrorKind::ApplyDecode("-".into()))
            .map_err(Error::from)
    } else {
        let reader = File::open(file).with_context(|_| ErrorKind::FsOpen(file.into()))?;
        serde_yaml::from_reader(reader)
            .with_context(|_| ErrorKind::ApplyDecode(file.into()))
            .map_err(Error::from)
    }
}

//! Inspect, manage and interact with RepliCore servers from a Command Line Interface.
use anyhow::Result;
use clap::Parser;

use replicore_client::Client;

mod cmd;
mod context;
mod formatter;
mod globals;
mod utils;

pub mod errors;

use self::cmd::Cli;
use self::globals::Globals;

/// Initialise an API client to interact with the control plane.
fn client(context: &self::context::Context) -> Result<Client> {
    let options = replicore_client::ClientOptions::url(&context.connection.url).client();
    // TODO: implement TLS config options when supported.
    let client = Client::with(options)?;
    Ok(client)
}

/// Initialise the replictl process and invoke a command implementation.
pub async fn run() -> Result<i32> {
    let cli = Cli::parse();
    let globals = Globals::initialise(cli).await;

    match &globals.cli.command {
        cmd::Command::Apply(cmd) => cmd::apply::run(&globals, cmd).await,
        cmd::Command::Cluster(cmd) => cmd::cluster_spec::run(&globals, cmd).await,
        cmd::Command::Context(cmd) => cmd::context::run(&globals, cmd).await,
        cmd::Command::Namespace(cmd) => cmd::namespace::run(&globals, cmd).await,
        cmd::Command::OAction(cmd) => cmd::oaction::run(&globals, cmd).await,
        cmd::Command::Platform(cmd) => cmd::platform::run(&globals, cmd).await,
    }
}

//mod errors;
//mod logging;

// Re-export errors so main can provide more accurate messages.
//pub use apiclient::ApiNotFound;
//pub use context::ScopeError;
//pub use errors::ContextNotFound;
//pub use errors::InvalidApply;

//use logging::LogOpt;

///// Replicante Core command line client.
//#[derive(Debug, Parser)]
//#[command(long_about = None)]
//#[command(version = VERSION)]
//pub struct Cli {
//    #[command(subcommand)]
//    pub command: Command,
//
//    #[command(flatten)]
//    pub context: ContextOpt,
//
//    #[command(flatten)]
//    pub log: LogOpt,
//}

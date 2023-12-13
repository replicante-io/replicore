//! Inspect, manage and interact with RepliCore servers from a Command Line Interface.
use anyhow::Result;
use clap::Parser;

mod cmd;
mod context;
mod formatter;
mod globals;
mod utils;

pub mod errors;

use self::cmd::Cli;
use self::globals::Globals;

/// Initialise the replictl process and invoke a command implementation.
pub async fn run() -> Result<i32> {
    let cli = Cli::parse();
    let globals = Globals::initialise(cli).await;

    match &globals.cli.command {
        cmd::Command::Context(cmd) => cmd::context::run(&globals, cmd).await,
    }
}

//mod apiclient;
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

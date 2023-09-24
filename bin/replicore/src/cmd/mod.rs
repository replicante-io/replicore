//! Built-in `replicore` commands.
use clap::Parser;
use clap::Subcommand;

pub mod server;
pub mod sync;

/// Replicante Core data store orchestration control plane.
#[derive(Debug, Parser)]
#[command(version, about)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Path to the Replicante Core configuration to use.
    #[arg(short = 'c', long = "config", default_value_t = String::from("replicore.yaml"))]
    pub config: String,

    /// Select the replicore command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Select the replicore command to run.
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Run the Replicante Core control plane server.
    #[command(alias = "run")]
    Server,

    /// Synchronise (initialise or migrate) stateful dependences so the server can run.
    #[command(alias = "sync-dependencies")]
    Sync,
}

use anyhow::Result;
use clap::Subcommand;
use slog::Logger;

mod get;
mod list;

/// Query and manage Namespace objects.
#[derive(Debug, Subcommand)]
pub enum Opt {
    /// Query information about a namespace.
    Get,

    /// List all Namespace objects in the system.
    List,
}

/// Execute the selected command.
pub async fn execute(logger: &Logger, cli: &crate::Cli, cmd_opt: &Opt) -> Result<i32> {
    match &cmd_opt {
        Opt::Get => get::execute(logger, cli).await,
        Opt::List => list::execute(logger, cli).await,
    }
}

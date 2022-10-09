use anyhow::Result;
use clap::Subcommand;
use slog::Logger;

mod change;
mod describe;
mod list;
mod login;
mod logout;
mod select;

/// Commands to operate on `replictl` contexts.
#[derive(Debug, Subcommand)]
pub enum Opt {
    /// Set or update an existing context's scope attributes.
    Change,

    /// Describe the active context.
    Describe,

    /// List known contexts.
    List,

    /// Connect to Replicante API server(s) or update connection details.
    Login,

    /// Forget how to connect to a Replicante API server and remove its context.
    Logout,

    /// Select a context to be the active context.
    Select,
}

/// Execute the selected command.
pub async fn execute(logger: &Logger, cli: &crate::Cli, context_cmd: &Opt) -> Result<i32> {
    match &context_cmd {
        Opt::Change => change::execute(logger, cli).await,
        Opt::Describe => describe::execute(logger, cli).await,
        Opt::List => list::execute(logger, cli).await,
        Opt::Login => login::execute(logger, cli).await,
        Opt::Logout => logout::execute(logger, cli).await,
        Opt::Select => select::execute(logger, cli).await,
    }
}

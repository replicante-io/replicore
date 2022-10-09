use anyhow::Result;
use clap::Args;
use clap::Subcommand;
use slog::Logger;

mod delete;
mod list;

/// Command line options common to all commands.
#[derive(Args, Debug)]
pub struct CommonOpt {
    /// Name of the DiscoverySettings object to operate on.
    #[arg(env = "RCTL_NAME", name = "NAME")]
    pub discovery_name: String,
}

/// Show and manage DiscoverySettings objects.
#[derive(Debug, Subcommand)]
pub enum Opt {
    /// Delete a DiscoverySettings object.
    Delete(CommonOpt),

    /// List all DiscoverySettings objects for the namespace.
    List,
}

/// Execute the selected command.
pub async fn execute(logger: &Logger, cli: &crate::Cli, cmd_opt: &Opt) -> Result<i32> {
    match &cmd_opt {
        Opt::Delete(delete_opt) => delete::execute(logger, cli, delete_opt).await,
        Opt::List => list::execute(logger, cli).await,
    }
}

use anyhow::Result;
use slog::Logger;
use structopt::StructOpt;

mod delete;
mod list;

// Command line options common to all commands.
// NOTE: this is not a docstring because StructOpt then uses it as the actions help.
#[derive(Debug, StructOpt)]
pub struct CommonOpt {
    /// Name of the DiscoverySettings object to operate on.
    #[structopt(env = "RCTL_NAME", name = "NAME")]
    pub discovery_name: String,
}

/// Show and manage DiscoverySettings objects.
#[derive(Debug, StructOpt)]
pub enum Opt {
    /// Delete a DiscoverySettings object.
    Delete(CommonOpt),

    /// List all DiscoverySettings objects for the namespace.
    List,
}

/// Execute the selected command.
pub async fn execute(logger: &Logger, opt: &crate::Opt, cmd_opt: &Opt) -> Result<i32> {
    match &cmd_opt {
        Opt::Delete(delete_opt) => delete::execute(logger, opt, delete_opt).await,
        Opt::List => list::execute(logger, opt).await,
    }
}

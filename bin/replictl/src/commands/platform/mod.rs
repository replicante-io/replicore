use anyhow::Result;
use clap::Args;
use clap::Subcommand;
use slog::Logger;

mod get;
mod list;

/// Command line options common to all commands.
#[derive(Args, Debug)]
pub struct CommonOpt {
    /// Name of the Platform object to operate on.
    #[arg(env = "RCTL_NAME", name = "NAME")]
    pub platform: String,
}

/// Query and manage Namespace objects.
#[derive(Debug, Subcommand)]
pub enum Opt {
    /// Query information about a namespace.
    Get(CommonOpt),

    /// List all Namespace objects in the system.
    List,
}

/// Execute the selected command.
pub async fn execute(logger: &Logger, cli: &crate::Cli, cmd_opt: &Opt) -> Result<i32> {
    match &cmd_opt {
        Opt::Get(opt) => get::execute(logger, cli, opt).await,
        Opt::List => list::execute(logger, cli).await,
    }
}

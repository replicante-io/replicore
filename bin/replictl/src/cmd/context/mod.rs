//! Manage configuration of RepliCore servers to access.
use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

use crate::Globals;

mod change;
mod configure;
mod delete;
mod list;
mod login;
mod select;
mod show;

/// Manage configuration of RepliCore servers to access.
#[derive(Debug, Parser)]
pub struct ContextCli {
    /// Select the `replictl context` command to run.
    #[command(subcommand)]
    pub command: ContextCmd,
}

/// Select the `replictl context` command to run.
#[derive(Debug, Subcommand)]
pub enum ContextCmd {
    /// Change scope attributes, such as namespace or cluster, of a context.
    Change,

    /// Configure or update the connection options for a RepliCore server.
    Configure,

    /// Ensure the selected context configuration is removed from.
    Delete,

    /// List known RepliCore servers.
    List,

    /// Placeholder for future Replicante Control Plane authentication logic.
    Login,

    /// Select the active context, the one used when none are specified.
    Select,

    /// Show details about the current `replictl` context.
    Show,
}

/// Execute the selected `replictl context` command.
pub async fn run(globals: &Globals, cmd: &ContextCli) -> Result<i32> {
    match cmd.command {
        ContextCmd::Change => self::change::run(globals).await,
        ContextCmd::Configure => self::configure::run(globals).await,
        ContextCmd::Delete => self::delete::run(globals).await,
        ContextCmd::List => self::list::run(globals).await,
        ContextCmd::Login => self::login::run(globals).await,
        ContextCmd::Select => self::select::run(globals).await,
        ContextCmd::Show => self::show::run(globals).await,
    }
}

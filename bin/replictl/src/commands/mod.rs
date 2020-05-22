use anyhow::Result;
use slog::Logger;
use structopt::StructOpt;

mod action;
mod apply;
mod cluster;
mod context;

use crate::Opt;

/// Command to execute.
#[derive(Debug, StructOpt)]
pub enum Command {
    /// Manage actions.
    Action(action::Opt),

    /// Apply changes as decribed by the YAML input (heavily inspired by https://kubernetes.io/).
    Apply(apply::Opt),

    /// Manage clusters.
    Cluster(cluster::Opt),

    /// Show or update replictl contexts.
    Context(context::Opt),
}

/// Execute the selected command.
pub async fn execute(logger: &Logger, opt: &Opt) -> Result<i32> {
    match &opt.command {
        Command::Action(action_opt) => action::execute(logger, opt, action_opt).await,
        Command::Apply(apply_opt) => apply::execute(logger, opt, apply_opt).await,
        Command::Cluster(cluster_opt) => cluster::execute(logger, opt, cluster_opt).await,
        Command::Context(context_opt) => context::execute(logger, opt, context_opt).await,
    }
}

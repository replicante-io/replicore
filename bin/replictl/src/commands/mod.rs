use anyhow::Result;
use clap::Subcommand;
use slog::Logger;

mod action;
mod apply;
mod cluster;
mod context;
mod discovery_settings;

use crate::Cli;

/// Command to execute.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show and manage actions.
    #[command(subcommand)]
    Action(action::Opt),

    /// Apply changes as described by the YAML input (heavily inspired by https://kubernetes.io/).
    Apply(apply::Opt),

    /// Show and manage clusters.
    #[command(subcommand)]
    Cluster(cluster::Opt),

    /// Show or update replictl contexts.
    #[command(subcommand)]
    Context(context::Opt),

    /// Show and manage DiscoverySettings objects.
    #[command(subcommand)]
    DiscoverySettings(discovery_settings::Opt),
}

/// Execute the selected command.
pub async fn execute(logger: &Logger, cli: &Cli) -> Result<i32> {
    match &cli.command {
        Command::Action(opt) => action::execute(logger, cli, opt).await,
        Command::Apply(apply_opt) => apply::execute(logger, cli, apply_opt).await,
        Command::Cluster(opt) => cluster::execute(logger, cli, opt).await,
        Command::Context(opt) => context::execute(logger, cli, opt).await,
        Command::DiscoverySettings(opt) => discovery_settings::execute(logger, cli, opt).await,
    }
}

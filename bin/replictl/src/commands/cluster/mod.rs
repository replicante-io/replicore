use anyhow::Result;
use clap::Subcommand;
use slog::Logger;

mod orchestrate;

/// Show and manage clusters.
#[derive(Debug, Subcommand)]
pub enum Opt {
    /// Schedule a cluster orchestration cycle.
    Orchestrate,
}

/// Execute the selected command.
pub async fn execute(logger: &Logger, cli: &crate::Cli, cluster_cmd: &Opt) -> Result<i32> {
    match &cluster_cmd {
        Opt::Orchestrate => orchestrate::execute(logger, cli).await,
    }
}

use anyhow::Result;
use slog::Logger;
use structopt::StructOpt;

mod orchestrate;

/// Show and manage clusters.
#[derive(Debug, StructOpt)]
pub enum Opt {
    /// Schedule a cluster orchestrateion cycle.
    Orchestrate,
}

/// Execute the selected command.
pub async fn execute(logger: &Logger, opt: &crate::Opt, cluster_cmd: &Opt) -> Result<i32> {
    match &cluster_cmd {
        Opt::Orchestrate => orchestrate::execute(logger, opt).await,
    }
}

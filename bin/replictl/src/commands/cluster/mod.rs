use anyhow::Result;
use slog::Logger;
use structopt::StructOpt;

mod refresh;

/// Commands to operate on actions.
#[derive(Debug, StructOpt)]
pub enum Opt {
    /// Schedule a cluster refresh.
    Refresh,
}

/// Execute the selected command.
pub async fn execute(logger: &Logger, opt: &crate::Opt, cluster_cmd: &Opt) -> Result<i32> {
    match &cluster_cmd {
        Opt::Refresh => refresh::execute(logger, opt).await,
    }
}

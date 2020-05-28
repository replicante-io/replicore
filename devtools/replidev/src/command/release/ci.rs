use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use tokio::process::Command;

use crate::error::ReleaseCheck;

const CI_RUN_ERROR: &str = "CI tests failed, please review the logs and fix any errors";

/// Run CI checks for the project to ensure they pass.
pub async fn run() -> Result<()> {
    let path = Path::new("ci/travis/build-script.sh");
    if !path.exists() {
        return Ok(());
    }

    println!("--> Running CI checks ...");
    let mut ci = Command::new(path);
    let status = ci
        .status()
        .await
        .context(CI_RUN_ERROR)
        .map_err(ReleaseCheck::from)?;
    if !status.success() {
        return ReleaseCheck::failed(anyhow::anyhow!(CI_RUN_ERROR));
    }
    Ok(())
}

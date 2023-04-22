use anyhow::Context;
use anyhow::Result;
use tokio::process::Command;

use crate::podman::Error;
use crate::Conf;

/// Execute a command in podman unshare environment (enter the user ns but not others).
pub async fn unshare(conf: &Conf, command: Vec<&str>) -> Result<()> {
    let status = Command::new(&conf.podman)
        .arg("unshare")
        .args(command)
        .status()
        .await
        .context(Error::ExecFailed)?;
    if !status.success() {
        let error = Error::CommandFailed(status.code().unwrap_or(-1));
        anyhow::bail!(error);
    }
    Ok(())
}

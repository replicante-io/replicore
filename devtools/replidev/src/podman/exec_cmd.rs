use anyhow::Context;
use anyhow::Result;
use tokio::process::Command;

use crate::Conf;
use crate::podman::Error;

/// Execute a command in a container.
pub async fn exec(conf: &Conf, name: &str, command: Vec<String>) -> Result<()> {
    let status = Command::new(&conf.podman)
        .arg("exec")
        .arg("-it")
        .arg(name)
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

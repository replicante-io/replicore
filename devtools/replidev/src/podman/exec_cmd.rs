use failure::ResultExt;
use tokio::process::Command;

use crate::Conf;
use crate::ErrorKind;
use crate::Result;

/// Execute a command in a container.
pub async fn exec(conf: &Conf, name: &str, command: Vec<String>) -> Result<()> {
    let status = Command::new(&conf.podman)
        .arg("exec")
        .arg("-it")
        .arg(name)
        .args(command)
        .status()
        .await
        .with_context(|_| ErrorKind::podman_exec("exec"))?;
    if !status.success() {
        let error = ErrorKind::podman_failed("exec");
        return Err(error.into());
    }
    Ok(())
}

use anyhow::Context;
use anyhow::Result;
use tokio::process::Command;

use crate::podman::Error;
use crate::Conf;

/// Stop AND REMOVE a pod matching the given name.
pub async fn pod_stop<S>(conf: &Conf, name: S) -> Result<()>
where
    S: std::fmt::Display,
{
    // Stop the pod first.
    println!("--> Stop pod {}", name);
    let status = Command::new(&conf.podman)
        .arg("pod")
        .arg("stop")
        .arg(name.to_string())
        .status()
        .await
        .context(Error::ExecFailed)?;
    if !status.success() {
        let error = Error::CommandFailed(status.code().unwrap_or(-1));
        return Err(error.into());
    }

    // Now remove the pod.
    println!("--> Remove pod {}", name);
    let status = Command::new(&conf.podman)
        .arg("pod")
        .arg("rm")
        .arg(name.to_string())
        .status()
        .await
        .context(Error::ExecFailed)?;
    if !status.success() {
        let error = Error::CommandFailed(status.code().unwrap_or(-1));
        return Err(error.into());
    }
    Ok(())
}

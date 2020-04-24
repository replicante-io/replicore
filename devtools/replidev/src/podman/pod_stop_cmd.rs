use failure::ResultExt;
use tokio::process::Command;

use crate::Conf;
use crate::ErrorKind;
use crate::Result;

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
        .with_context(|_| ErrorKind::podman_exec("pod stop"))?;
    if !status.success() {
        let error = ErrorKind::podman_failed("pod stop");
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
        .with_context(|_| ErrorKind::podman_exec("pod rm"))?;
    if !status.success() {
        let error = ErrorKind::podman_failed("pod rm");
        return Err(error.into());
    }
    Ok(())
}

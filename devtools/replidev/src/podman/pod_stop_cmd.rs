use std::process::Command;

use failure::ResultExt;

use crate::ErrorKind;
use crate::Result;

/// Stop AND REMOVE a pod matching the given name.
pub fn pod_stop<S>(name: S) -> Result<()>
where
    S: std::fmt::Display,
{
    // Stop the pod first.
    println!("--> Stop pod {}", name);
    let status = Command::new("podman")
        .arg("pod")
        .arg("stop")
        .arg(name.to_string())
        .status()
        .with_context(|_| ErrorKind::podman_exec("pod stop"))?;
    if !status.success() {
        let error = ErrorKind::podman_failed("pod stop");
        return Err(error.into());
    }

    // Now remove the pod.
    println!("--> Remove pod {}", name);
    let status = Command::new("podman")
        .arg("pod")
        .arg("rm")
        .arg(name.to_string())
        .status()
        .with_context(|_| ErrorKind::podman_exec("pod rm"))?;
    if !status.success() {
        let error = ErrorKind::podman_failed("pod rm");
        return Err(error.into());
    }
    Ok(())
}

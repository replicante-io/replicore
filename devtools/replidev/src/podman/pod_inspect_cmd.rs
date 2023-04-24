use anyhow::Context;
use anyhow::Result;
use tokio::process::Command;

use crate::podman::Error;
use crate::Conf;

/// Return the output of inspecting a pod.
pub async fn pod_inspect(conf: &Conf, pod_id: &str) -> Result<Vec<u8>> {
    let mut podman = Command::new(&conf.podman);
    podman
        .stderr(std::process::Stdio::inherit())
        .arg("pod")
        .arg("inspect")
        .arg(pod_id);
    let output = podman.output().await.context(Error::ExecFailed)?;
    if !output.status.success() {
        let error = Error::CommandFailed(output.status.code().unwrap_or(-1));
        return Err(error.into());
    }
    Ok(output.stdout)
}

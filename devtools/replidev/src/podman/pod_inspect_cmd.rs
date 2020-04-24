use failure::ResultExt;
use tokio::process::Command;

use crate::Conf;
use crate::ErrorKind;
use crate::Result;

/// Return the output of inspecting a pod.
pub async fn pod_inspect(conf: &Conf, pod_id: &str) -> Result<Vec<u8>> {
    let mut podman = Command::new(&conf.podman);
    podman
        .stderr(std::process::Stdio::inherit())
        .arg("pod")
        .arg("inspect")
        .arg(pod_id);
    let output = podman
        .output()
        .await
        .with_context(|_| ErrorKind::podman_exec("pod inspect"))?;
    if !output.status.success() {
        let error = ErrorKind::podman_failed("pod inspect");
        return Err(error.into());
    }
    Ok(output.stdout)
}

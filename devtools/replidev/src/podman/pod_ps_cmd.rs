use anyhow::Context;
use anyhow::Result;
use tokio::process::Command;

use crate::podman::Error;
use crate::Conf;

/// Start a pod matching the given definition.
pub async fn pod_ps<'a, F>(conf: &Conf, format: F, filters: Vec<&str>) -> Result<Vec<u8>>
where
    F: Into<Option<&'a str>>,
{
    let mut podman = Command::new(&conf.podman);
    podman
        .stderr(std::process::Stdio::inherit())
        .arg("pod")
        .arg("ps");
    if let Some(format) = format.into() {
        podman.arg("--format").arg(format);
    }
    if !filters.is_empty() {
        let filters = filters.join(",");
        podman.arg("--filter").arg(filters);
    }
    let output = podman.output().await.context(Error::ExecFailed)?;
    if !output.status.success() {
        let error = Error::CommandFailed(output.status.code().unwrap_or(-1));
        anyhow::bail!(error);
    }
    Ok(output.stdout)
}

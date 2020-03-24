use std::process::Command;

use failure::ResultExt;

use crate::Conf;
use crate::ErrorKind;
use crate::Result;

/// Start a pod matching the given definition.
pub fn pod_ps<'a, F>(conf: &Conf, format: F, filters: Vec<&str>) -> Result<Vec<u8>>
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
    for filter in filters {
        podman.arg("--filter").arg(filter);
    }
    let output = podman
        .output()
        .with_context(|_| ErrorKind::podman_exec("pod ps"))?;
    if !output.status.success() {
        let error = ErrorKind::podman_failed("pod ps");
        return Err(error.into());
    }
    Ok(output.stdout)
}

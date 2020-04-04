use std::process::Command;

use failure::ResultExt;

use crate::Conf;
use crate::ErrorKind;
use crate::Result;

/// TODO
pub fn pod_inspect(conf: &Conf, pod_id: &str) -> Result<Vec<u8>> {
    let mut podman = Command::new(&conf.podman);
    podman
        .stderr(std::process::Stdio::inherit())
        .arg("pod")
        .arg("inspect")
        .arg(pod_id);
    let output = podman
        .output()
        .with_context(|_| ErrorKind::podman_exec("pod inspect"))?;
    if !output.status.success() {
        let error = ErrorKind::podman_failed("pod inspect");
        return Err(error.into());
    }
    Ok(output.stdout)
}

use std::process::Command;

use failure::ResultExt;

use crate::ErrorKind;
use crate::Result;

/// Execute a command in a container.
pub fn exec(name: &str, command: Vec<String>) -> Result<()> {
    let status = Command::new("podman")
        .arg("exec")
        .arg("-it")
        .arg(name)
        .args(command)
        .status()
        .with_context(|_| ErrorKind::podman_exec("exec"))?;
    if !status.success() {
        let error = ErrorKind::podman_failed("exec");
        return Err(error.into());
    }
    Ok(())
}

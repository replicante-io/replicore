use std::process::Command;

use failure::ResultExt;

use crate::ErrorKind;
use crate::Result;

/// Execute a command in podman unshare environment (enter the user ns but not others).
pub fn unshare(command: Vec<&str>) -> Result<()> {
    let status = Command::new("podman")
        .arg("unshare")
        .args(command)
        .status()
        .with_context(|_| ErrorKind::podman_exec("unshare"))?;
    if !status.success() {
        let error = ErrorKind::podman_failed("unshare");
        return Err(error.into());
    }
    Ok(())
}

use anyhow::Context;
use anyhow::Result;
use tokio::process::Command;

use crate::conf::Conf;

/// Copy files out of a running container.
pub async fn copy(conf: &Conf, from: &str, to: &str) -> Result<()> {
    // Prepare the copy command.
    let mut podman = Command::new(&conf.podman);
    podman.arg("cp").arg(from).arg(to);

    // Copy the file.
    let status = podman
        .status()
        .await
        .with_context(|| format!("Failed to copy {} from {}", to, from))?;
    if !status.success() {
        anyhow::bail!("Failed to copy {} from {}", to, from);
    }
    Ok(())
}

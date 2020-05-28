use anyhow::Context;
use anyhow::Result;
use tokio::process::Command;

use crate::conf::Conf;

/// Stop a running container with the given name.
pub async fn stop(conf: &Conf, name: &str) -> Result<()> {
    // Prepare the stop command.
    let mut podman = Command::new(&conf.podman);
    podman.arg("stop").arg(name);

    // Stop the container.
    let status = podman
        .status()
        .await
        .with_context(|| format!("Failed to stop container {}", name))?;
    if !status.success() {
        anyhow::bail!("Failed to stop container {}", name);
    }
    Ok(())
}

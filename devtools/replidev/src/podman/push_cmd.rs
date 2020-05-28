use anyhow::Context;
use anyhow::Result;
use tokio::process::Command;

use crate::conf::Conf;

/// Push the given container image tag.
pub async fn push(conf: &Conf, tag: &str) -> Result<()> {
    let mut podman = Command::new(&conf.podman);
    podman.arg("push").arg(tag);
    let status = podman
        .status()
        .await
        .with_context(|| format!("Failed to push {}", tag))?;
    if !status.success() {
        anyhow::bail!("Failed to push {}", tag);
    }
    Ok(())
}

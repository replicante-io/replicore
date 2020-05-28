use anyhow::Context;
use anyhow::Result;
use tokio::process::Command;

use crate::conf::Conf;

/// Pull an image.
pub async fn pull(conf: &Conf, tag: &str) -> Result<()> {
    let mut podman = Command::new(&conf.podman);
    podman.arg("pull").arg(tag);
    let status = podman
        .status()
        .await
        .with_context(|| format!("Failed to pull {}", tag))?;
    if !status.success() {
        anyhow::bail!("Failed to pull {}", tag);
    }
    Ok(())
}

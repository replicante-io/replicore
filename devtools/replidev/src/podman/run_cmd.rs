use anyhow::Context;
use anyhow::Result;
use tokio::process::Command;

use crate::conf::Conf;

/// Run a container with the given options and arguments.
pub async fn run(conf: &Conf, tag: &str, name: &str, opt: &[&str], args: &[&str]) -> Result<()> {
    // Prepare the run command.
    let mut podman = Command::new(&conf.podman);
    podman
        .arg("run")
        .arg("--name")
        .arg(name)
        .args(opt)
        .arg(tag)
        .args(args);

    // Run the container.
    let status = podman
        .status()
        .await
        .with_context(|| format!("Failed to run {}", name))?;
    if !status.success() {
        anyhow::bail!("Failed to run {}", name);
    }
    Ok(())
}

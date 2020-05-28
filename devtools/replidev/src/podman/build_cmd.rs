use anyhow::Result;
use tokio::process::Command;

use crate::conf::Conf;
use crate::conf::Image;
use crate::error::ReleaseCheck;

/// Build a container image from the configured specification.
pub async fn build(conf: &Conf, image: &Image, use_cache: bool, tags: Vec<String>) -> Result<()> {
    // Prepare the build command.
    let mut podman = Command::new(&conf.podman);
    podman.arg("build").arg("--force-rm").arg("--format=docker");
    if !use_cache {
        podman.arg("--no-cache");
    }
    for tag in tags {
        println!("--> Tagging image with: {}", tag);
        podman.arg("--tag").arg(tag);
    }
    if let Some(dockerfile) = &image.dockerfile {
        podman.arg("--file").arg(dockerfile);
    }
    podman.arg(&image.context);

    // Build the image.
    let status = podman.status().await.map_err(|error| {
        let error = anyhow::anyhow!(error);
        let error = error.context(format!("Failed to build an image for {}", image.name));
        ReleaseCheck::from(error)
    })?;
    if status.success() {
        Ok(())
    } else {
        ReleaseCheck::failed(anyhow::anyhow!(
            "Failed to build an image for {}",
            image.name
        ))
    }
}

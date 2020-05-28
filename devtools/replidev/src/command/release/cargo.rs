use std::process::ExitStatus;

use anyhow::Context;
use anyhow::Result;
use tokio::process::Command;

use crate::command::cargo::Opt;
use crate::conf::Conf;
use crate::conf::CratesPublish;
use crate::error::ReleaseCheck;

const PUBLISH_INTERACT_ERROR: &str = "Unable to confirm that a crate should be published";

/// Run 'cargo publish' for a crate to publish it.
pub async fn publish(publish: &CratesPublish) -> Result<()> {
    println!(
        "--> Running 'cargo publish --manifest-path {}'",
        publish.path
    );

    // Confirm publishing each crate so unchanged crates can be skipped.
    let confirmed = tokio::task::spawn_blocking(|| {
        dialoguer::Confirm::new()
            .default(false)
            .with_prompt("Do NOT publish this crate unless it has changed. Publish crate?")
            .interact()
    })
    .await
    .context(PUBLISH_INTERACT_ERROR)?
    .context(PUBLISH_INTERACT_ERROR)?;
    if !confirmed {
        println!("--> Skipping crate at {}", publish.path);
        return Ok(());
    }

    // Call cargo publish to publish the crate.
    // This time each crate must successfully be pushed.
    let status = run_publish(publish, false, false).await?;
    if !status.success() {
        anyhow::bail!("Failed to publish crate at {}", publish.path);
    }
    Ok(())
}

/// Run 'cargo publish --dry-run' for a crate that will be published later.
pub async fn publish_check(publish: &CratesPublish, allow_dirty: bool) -> Result<()> {
    println!(
        "--> Running 'cargo publish --dry-run --manifest-path {}'",
        publish.path
    );
    let status = run_publish(publish, allow_dirty, true)
        .await
        .map_err(ReleaseCheck::from)?;
    if !status.success() && !publish.may_fail_check {
        return ReleaseCheck::failed(anyhow::anyhow!(
            "Failed to test publishing crate at {}",
            publish.path
        ));
    }
    if !status.success() && publish.may_fail_check {
        println!(
            "--> WARN: Failed to test publishing crate at {}",
            publish.path
        );
        println!("--> WARN: This may be because of dependencies to crates that still need to be published.");
        println!("--> WARN: Check every error CAREFULLY before proceeding with the release");
    }
    Ok(())
}

/// Run cargo publish to verify or publish a crate.
async fn run_publish(
    publish: &CratesPublish,
    allow_dirty: bool,
    dry_run: bool,
) -> Result<ExitStatus> {
    let mut cargo = Command::new("cargo");
    cargo
        .arg("publish")
        .arg("--manifest-path")
        .arg(&publish.path);
    if dry_run {
        cargo.arg("--dry-run");
    }
    if allow_dirty {
        cargo.arg("--allow-dirty");
    }
    let status = cargo.status().await.with_context(|| {
        format!(
            "Failed to execute cargo publish command for crate at {}",
            publish.path
        )
    })?;
    Ok(status)
}

/// Update all cargo workspaces and notify users of available updates.
pub async fn update(conf: &Conf) -> Result<()> {
    if !conf.crates.workspaces.is_empty() {
        crate::command::cargo::run(Opt::update(), conf).await?;
        crate::command::cargo::run(Opt::outdated(), conf).await?;
    }
    Ok(())
}

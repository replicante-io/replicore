use anyhow::Context;
use anyhow::Result;
use tokio::process::Command;

use crate::conf::Conf;

/// Update all npm workspaces and notify users of available updates.
pub async fn update(conf: &Conf) -> Result<()> {
    for workspace in &conf.npm.workspaces {
        run_npm_update(workspace).await?;
    }
    for workspace in &conf.npm.workspaces {
        run_npm_outdated(workspace).await?;
    }
    Ok(())
}

// Run npm outdated for each given package, ignoring the exit code.
async fn run_npm_outdated(workspace: &str) -> Result<()> {
    println!("--> Running npm outdated for {}", workspace);
    let mut npm = Command::new("npm");
    npm.arg("outdated")
        .arg("--progress=true")
        .current_dir(workspace);
    npm.status()
        .await
        .with_context(|| format!("Failed to execute npm outdated for workspace {}", workspace))?;
    Ok(())
}

// Run npm update for each given package.
async fn run_npm_update(workspace: &str) -> Result<()> {
    println!("--> Running npm update for {}", workspace);
    let mut npm = Command::new("npm");
    npm.arg("update")
        .arg("--progress=true")
        .current_dir(workspace);
    let status = npm
        .status()
        .await
        .with_context(|| format!("Failed to execute npm update for workspace {}", workspace))?;
    if !status.success() {
        anyhow::bail!("Failed to execute npm update for workspace {}", workspace);
    }
    Ok(())
}

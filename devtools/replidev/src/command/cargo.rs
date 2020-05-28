use anyhow::Context;
use anyhow::Result;
use structopt::StructOpt;
use tokio::process::Command;

use crate::conf::Conf;

#[derive(Debug, StructOpt)]
pub struct Opt {
    /// The cargo command to run.
    pub command: String,

    /// Arguments to the cargo command to run.
    #[structopt(last = true)]
    pub arguments: Vec<String>,
}

impl Opt {
    /// Options to run a cargo outdated --depth 1 command.
    pub fn outdated() -> Opt {
        Opt {
            command: "outdated".to_string(),
            arguments: vec!["--depth=1".to_string()],
        }
    }

    /// Options to run a cargo update command.
    pub fn update() -> Opt {
        Opt {
            command: "update".to_string(),
            arguments: Vec::new(),
        }
    }
}

/// Configuration related commands.
pub async fn run(opt: Opt, conf: &Conf) -> Result<i32> {
    let workspaces = &conf.crates.workspaces;
    if workspaces.is_empty() {
        anyhow::bail!("Need a set of workspaces configured for replidev cargo to operate on");
    }
    for workspace in workspaces {
        println!(
            "--> Running 'cargo {} --manifest-path {}' with the provided arguments",
            opt.command, workspace,
        );
        run_cargo_command(&opt, workspace).await?;
    }
    Ok(0)
}

async fn run_cargo_command(opt: &Opt, workspace: &str) -> Result<()> {
    let mut cargo = Command::new("cargo");
    cargo
        .arg(&opt.command)
        .arg("--manifest-path")
        .arg(workspace)
        .args(&opt.arguments);
    let status = cargo.status().await.with_context(|| {
        format!(
            "Failed to execute cargo {} for workspace {}",
            opt.command, workspace,
        )
    })?;
    if !status.success() {
        anyhow::bail!(
            "Failed to execute cargo {} for workspace {}",
            opt.command,
            workspace,
        );
    }
    Ok(())
}

use anyhow::Result;
use structopt::StructOpt;

use crate::conf::Conf;
use crate::error::InvalidProject;

mod changes;
mod git;

const STAGE_CHANGES: &str = "changes";

lazy_static::lazy_static! {
    static ref ALL_STAGES: Vec<&'static str> = vec![
        STAGE_CHANGES,
    ];
}

#[derive(Debug, StructOpt)]
pub struct CheckOpt {
    /// Allow dirty working directories to be packaged.
    #[structopt(long = "allow-dirty")]
    pub allow_dirty: bool,

    /// Git tag to use as reference for the previous release.
    #[structopt(long = "git-start")]
    pub git_start: Option<String>,

    /// Select the check stage(s) to run, to iterate over specific tasks more quickly.
    #[structopt(
        long = "stage", name = "stage",
        possible_values = ALL_STAGES.as_slice(),
    )]
    pub stages: Vec<String>,
}

/// Release related commands.
#[derive(Debug, StructOpt)]
pub enum Opt {
    /// Check the project readiness for release.
    #[structopt(name = "check")]
    Check(CheckOpt),
}

pub async fn run(opt: Opt, conf: Conf) -> Result<i32> {
    if !conf.project.allow_release() {
        anyhow::bail!(InvalidProject::new(conf.project, "release"));
    }
    match opt {
        Opt::Check(opt) => check(opt, conf).await,
    }
}

/// Run through project release check tasks.
async fn check(opt: CheckOpt, conf: Conf) -> Result<i32> {
    // Always check if git is clean.
    git::check_clean(&opt).await?;
    let git_start = match opt.git_start.clone() {
        Some(start) => start,
        None => git::find_most_recent_tag().await?,
    };

    // Figure out which stages the user has asked for.
    let stages = if opt.stages.is_empty() {
        ALL_STAGES.clone()
    } else {
        opt.stages.iter().map(String::as_str).collect()
    };

    // Execute the selected checks, in order.
    let mut issues = crate::error::ReleaseCheck::new();
    if stages.contains(&STAGE_CHANGES) {
        println!("Looking for crates that changed since last release ...");
        issues.check(changes::check_crates(&opt, &conf, &git_start).await)?;
    }

    // Report on the outcome of the checks.
    issues.into_result().map(|_| 0)
}

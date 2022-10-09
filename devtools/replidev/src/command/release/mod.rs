use anyhow::Context;
use anyhow::Result;
use clap::Args;
use clap::Subcommand;
use clap::ValueEnum;

use crate::conf::Conf;
use crate::error::InvalidProject;
use crate::error::ReleaseCheck;

mod cargo;
mod changes;
mod ci;
mod git;
mod images;
mod npm;
mod version;

const STEP_INTERACT_ERROR: &str = "Unable to confirm that a required step was performed";

lazy_static::lazy_static! {
    static ref CHECK_TASKS: Vec<CheckTasks> = vec![
        CheckTasks::Ci,
        CheckTasks::Images,
        CheckTasks::Packages,
    ];
    static ref PREP_TASKS: Vec<PrepTasks> = vec![
        PrepTasks::Changes,
        PrepTasks::Updates,
    ];
    static ref PUBLISH_TASKS: Vec<PublishTasks> = vec![
        PublishTasks::Images,
        PublishTasks::Packages,
        PublishTasks::Tag,
        PublishTasks::Binaries,
    ];
}

#[derive(Args, Debug)]
pub struct CheckOpt {
    /// Allow dirty working directories to be packaged.
    #[arg(long = "allow-dirty")]
    pub allow_dirty: bool,

    /// Git tag to use as reference for the previous release.
    #[arg(long = "git-start")]
    pub git_start: Option<String>,

    /// Check tasks to perform.
    #[command(flatten)]
    pub tasks: TasksArg<CheckTasks>,
}

/// Possible tasks performed by `replidev release check`.
#[derive(Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum CheckTasks {
    Ci,
    Images,
    Packages,
}

#[derive(Args, Debug)]
pub struct PrepOpt {
    /// Git tag to use as reference for the previous release.
    #[arg(long = "git-start")]
    pub git_start: Option<String>,

    /// Prep tasks to perform.
    #[command(flatten)]
    pub tasks: TasksArg<PrepTasks>,
}

/// Possible tasks performed by `replidev release prep`.
#[derive(Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum PrepTasks {
    Changes,
    Updates,
}

#[derive(Args, Debug)]
pub struct PublishOpt {
    /// Don't pull docker images before extracting binaries.
    #[arg(long = "skip-pull")]
    pub skip_pull: bool,

    /// Publish tasks to perform.
    #[command(flatten)]
    pub tasks: TasksArg<PublishTasks>,
}

/// Possible tasks performed by `replidev release publish`.
#[derive(Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum PublishTasks {
    Images,
    Packages,
    #[value(name = "git-tag")]
    Tag,
    #[value(name = "collect-binaries")]
    Binaries,
}

/// Release related commands.
#[derive(Debug, Subcommand)]
pub enum Opt {
    /// Check the project readiness for release.
    #[command(name = "check")]
    Check(CheckOpt),

    /// Performs pre-release tasks for the project.
    #[command(name = "prep")]
    Prep(PrepOpt),

    /// Build and publish artifacts for the current version of the project.
    #[command(name = "publish")]
    Publish(PublishOpt),
}

/// Generic argument to select specific tasks to run.
#[derive(Args, Debug)]
pub struct TasksArg<T: ValueEnum + Send + Sync + 'static> {
    /// Select the task(s) to perform (can be specified multiple times).
    #[arg(long = "task", name = "TASK", value_enum)]
    pub tasks: Vec<T>,
}

impl<T: ValueEnum + Send + Sync + 'static> std::ops::Deref for TasksArg<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.tasks
    }
}

pub async fn run(opt: Opt, conf: Conf) -> Result<i32> {
    if !conf.project.allow_release() {
        anyhow::bail!(InvalidProject::new(conf.project, "release"));
    }
    match opt {
        Opt::Check(opt) => check(&opt, &conf).await,
        Opt::Prep(opt) => prep(&opt, &conf).await,
        Opt::Publish(opt) => publish(&opt, &conf).await,
    }
}

/// Run through project release check tasks.
async fn check(opt: &CheckOpt, conf: &Conf) -> Result<i32> {
    // Always check if git is clean.
    git::check_clean(opt.allow_dirty).await?;
    confirm_step_with_user("prep").await?;

    // Figure out which tasks the user has asked for.
    let tasks = if opt.tasks.is_empty() {
        std::ops::Deref::deref(&CHECK_TASKS)
    } else {
        std::ops::Deref::deref(&opt.tasks)
    };
    let mut issues = crate::error::ReleaseCheck::new();

    // Run the CI script to ensure all checks pass.
    if tasks.contains(&CheckTasks::Ci) {
        issues.check(ci::run().await)?;
    }

    // Check that container images build correctly.
    if tasks.contains(&CheckTasks::Images) {
        issues.check(images::build_for_check(conf).await)?;
    }

    // Check packages configured to be published.
    if tasks.contains(&CheckTasks::Packages) {
        for publish in &conf.crates.publish {
            issues.check(cargo::publish_check(publish, opt.allow_dirty).await)?;
        }
    }

    // Report on the outcome of the checks.
    issues.into_result().map(|_| 0)
}

/// Confirm the user successfully run a replidev release "step".
async fn confirm_step_with_user(step: &'static str) -> Result<()> {
    let confirmed = tokio::task::spawn_blocking(move || {
        dialoguer::Confirm::new()
            .default(false)
            .with_prompt(format!(
                "Have you successfully run replidev release {} already?",
                step,
            ))
            .interact()
    })
    .await
    .context(STEP_INTERACT_ERROR)?
    .context(STEP_INTERACT_ERROR)?;
    if confirmed {
        return Ok(());
    }
    ReleaseCheck::failed(anyhow::anyhow!(
        "You must first successfully run replidev release {}",
        step,
    ))
}

/// Performs pre-release tasks for the project.
async fn prep(opt: &PrepOpt, conf: &Conf) -> Result<i32> {
    // Figure out which tasks the user has asked for.
    let tasks = if opt.tasks.is_empty() {
        std::ops::Deref::deref(&PREP_TASKS)
    } else {
        std::ops::Deref::deref(&opt.tasks)
    };
    let mut issues = crate::error::ReleaseCheck::new();

    // Remind the user to update changelogs and versions.
    if tasks.contains(&PrepTasks::Changes) {
        let git_start = match opt.git_start.clone() {
            Some(start) => start,
            None => git::find_most_recent_tag().await?,
        };
        issues.check(changes::check_crates(conf, &git_start).await)?;
        issues.check(changes::check_npm(conf, &git_start).await)?;
    }

    // Update packages for all workspaces.
    if tasks.contains(&PrepTasks::Updates) {
        issues.check(cargo::update(conf).await)?;
        issues.check(npm::update(conf).await)?;
    }

    issues.into_result().map(|_| 0)
}

/// Build and publish artifacts for the current version of the project.
async fn publish(opt: &PublishOpt, conf: &Conf) -> Result<i32> {
    // Before we spend time working and push some artifacts ensure we are configured.
    conf.release_tag.as_ref().ok_or_else(|| {
        anyhow::anyhow!("A release_tag must be configured for replidev release publish to work")
    })?;

    // Always check if git is clean and don't allow skipping.
    git::check_clean(false).await?;
    confirm_step_with_user("prep").await?;
    confirm_step_with_user("check").await?;

    // Figure out which tasks the user has asked for.
    let tasks = if opt.tasks.is_empty() {
        std::ops::Deref::deref(&PUBLISH_TASKS)
    } else {
        std::ops::Deref::deref(&opt.tasks)
    };

    // Build and push container images.
    if tasks.contains(&PublishTasks::Images) {
        images::build_for_publish(conf).await?;
        images::push(conf).await?;
    }

    // Publish packages that need publishing.
    if tasks.contains(&PublishTasks::Packages) {
        for publish in &conf.crates.publish {
            cargo::publish(publish).await?;
        }
    }

    // Tag the repo with the release and push all tags.
    if tasks.contains(&PublishTasks::Tag) {
        git::tag(conf).await?;
    }

    // Collect pre-built binaries.
    if tasks.contains(&PublishTasks::Binaries) {
        images::extract_binaries(conf, opt.skip_pull).await?;
    }

    // Finally all done!
    println!("--> RELEASE PROCESS COMPLETE!");
    println!("--> The last thing left is to create a release in GitHub and attach any pre-built binaries to it");
    println!("--> Don't forget to push your tags when you push your release commit");
    Ok(0)
}

use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use chrono::Utc;
use git2::DiffOptions;
use git2::Repository;
use git2::StatusOptions;
use git2::Tree;
use tokio::process::Command;

use super::version;
use crate::conf::Conf;
use crate::conf::ReleaseTag;
use crate::error::ReleaseCheck;

const GIT_TAG_ERROR: &str = "Failed to tag the current git commit";

/// Detect any changes in the given path since the given tree.
pub fn changes_in_path<'repo>(
    repo: &'repo Repository,
    tree: &Tree<'repo>,
    path: &Path,
) -> Result<bool> {
    let mut options = DiffOptions::new();
    options.max_size(-1).pathspec(path.as_os_str());
    let diff = repo
        .diff_tree_to_workdir_with_index(Some(tree), Some(&mut options))
        .with_context(|| format!("Failed to generate git diff for '{:?}'", path))?
        .stats()
        .with_context(|| format!("Failed to process git diff for '{:?}'", path))?;
    Ok(diff.files_changed() > 0)
}

/// Check if the git working directory is clean.
pub async fn check_clean(allow_dirty: bool) -> Result<()> {
    // Figure out state of the repo.
    let repo = Repository::discover(".").expect("unable to find repository");
    let mut options = StatusOptions::new();
    options
        .include_untracked(true)
        .include_ignored(false)
        .include_unmodified(false)
        .exclude_submodules(true)
        .sort_case_insensitively(true);
    let statuses = repo
        .statuses(Some(&mut options))
        .expect("unable to list repository changes");

    // Determine if the repo is clean or not.
    let clean = statuses.iter().next().is_none();
    if clean {
        return Ok(());
    }

    // Warn if the user --allow-dirty or fail if not.
    if allow_dirty {
        eprintln!("Not all changes are committed to git but --allow-dirty was set.");
        return Ok(());
    }
    let error = anyhow::anyhow!("Not all changes are committed to git");
    ReleaseCheck::failed(error)
}

/// Find the most recent tag for the repo.
pub async fn find_most_recent_tag() -> Result<String> {
    let tag = tokio::task::spawn_blocking(|| -> Result<String> {
        let repo = Repository::discover(".").expect("unable to find repository");
        let tags = repo.tag_names(None).context("Failed to list tags")?;
        let mut times = Vec::new();
        for name in tags.iter() {
            let name = name.expect("tag must have a name");
            let obj = repo.revparse_single(name)?;
            let commit = obj
                .peel_to_commit()
                .with_context(|| format!("Tag {} does not point to a commit", name))?;
            let time = commit.time().seconds();
            times.push((time, name.to_string()));
        }
        times.sort();
        match times.pop() {
            Some((_, tag)) => Ok(tag),
            None => anyhow::bail!("No tags found in the repository"),
        }
    })
    .await
    .context("Failed to find most recent tag")??;
    Ok(tag)
}

/// Tag the current commit with the release version determined based on the configuration.
pub async fn tag(conf: &Conf) -> Result<()> {
    // We check this is set at the beginning of replidev release publish.
    let source = conf.release_tag.as_ref().unwrap();
    let version = match source {
        ReleaseTag::Cargo { path } => {
            let version = version::cargo(&path).await?;
            format!("v{}", version)
        }
        ReleaseTag::Date => Utc::now().format("%Y-%m-%d").to_string(),
        ReleaseTag::Npm { path } => {
            let version = version::npm(&path).await?;
            format!("v{}", version)
        }
    };

    // Call git to create the tag.
    println!("--> Tagging the current commit as {}", version);
    let mut git = Command::new("git");
    git.arg("tag").arg(version);
    let status = git.status().await.context(GIT_TAG_ERROR)?;
    if !status.success() {
        anyhow::bail!(GIT_TAG_ERROR);
    }
    Ok(())
}

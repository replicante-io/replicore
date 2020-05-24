use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use git2::DiffOptions;
use git2::Repository;
use git2::StatusOptions;
use git2::Tree;

use super::CheckOpt;
use crate::error::ReleaseCheck;

/// Detect any changes in the given path since the given tree.
pub fn changes_in_path<'repo>(
    repo: &'repo Repository,
    tree: &Tree<'repo>,
    path: &PathBuf,
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
pub async fn check_clean(args: &CheckOpt) -> Result<()> {
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
    if args.allow_dirty {
        eprintln!("Not all git changes are committed but --allow-dirty was set.");
        eprintln!("Results may not be accurrate.");
        return Ok(());
    }
    let error = anyhow::anyhow!("Not all git changes are committed");
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

use std::io::ErrorKind;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use git2::Repository;
use glob::glob;

use super::CheckOpt;
use crate::conf::Conf;
use crate::error::ReleaseCheck;

const CRATE_LIST_ERROR: &str = "Failed to list crates";

/// Check changes to crates since --git-start to remind developers to
/// update changelogs and versions..
pub async fn check_crates(_: &CheckOpt, conf: &Conf, git_start: &str) -> Result<()> {
    // Find crates by looking for Cargo.toml files.
    let ignored_crates = conf
        .ignored_crates
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let crates = tokio::task::spawn_blocking(move || -> Result<_> {
        let mut crates = Vec::new();
        for entry in glob("**/Cargo.toml").context(CRATE_LIST_ERROR)? {
            let mut entry = match entry {
                Err(error) if error.error().kind() == ErrorKind::PermissionDenied => continue,
                Ok(path) if ignored_crates.contains(&path) => continue,
                entry => entry.context(CRATE_LIST_ERROR)?,
            };
            entry.pop();
            crates.push(entry);
        }
        Ok(crates)
    })
    .await
    .context(CRATE_LIST_ERROR)??;

    // Check each crate for git changes.
    let repo = Repository::discover(".").expect("unable to find repository");
    let start = repo
        .revparse_single(git_start)
        .context("Failed to parse starting git revision")?;
    let start = start
        .peel_to_tree()
        .context("Provided starting git revision does not point to a tree")?;

    let mut changes = Vec::new();
    for path in crates {
        let changed = super::git::changes_in_path(&repo, &start, &path)?;
        if changed {
            changes.push(path);
        }
    }

    // If any crate changed, confirm the changes with the user.
    if changes.is_empty() {
        eprintln!("Looks like nothing changed since {}", git_start);
        return Ok(());
    }

    println!(
        "It looks like the following crates were changed since {}",
        git_start
    );
    for path in changes {
        println!("  * {}", path.to_string_lossy());
    }
    let confirmed = dialoguer::Confirm::new()
        .default(false)
        .with_prompt("Have all changelogs and versions been updated?")
        .interact()
        .context("Unable to confirm changes were made by the user")?;
    if confirmed {
        return Ok(());
    }
    ReleaseCheck::failed(anyhow::anyhow!(
        "You must confirm that all changelogs and versions were updated"
    ))
}

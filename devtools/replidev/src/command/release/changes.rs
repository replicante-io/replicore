use std::io::ErrorKind;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use git2::Repository;

use crate::conf::Conf;
use crate::error::ReleaseCheck;

const FILE_LIST_ERROR: &str = "Failed to scan the repository for files";
const USER_INTERACT_ERROR: &str = "Unable to confirm changes to changelogs and versions were made";

/// Check changes to crates since git_start to remind developers to
/// update changelogs and versions.
pub async fn check_crates(conf: &Conf, git_start: &str) -> Result<()> {
    if !conf.project.search_for_crates() {
        return Ok(());
    }
    println!("--> Looking for crates that changed since last release ...");

    // Find crates by looking for Cargo.toml files
    // then check each crate for git changes.
    let crates = files_by_name("Cargo.toml", conf.crates.ignored.as_slice()).await?;
    let changes = look_for_changes(&crates, git_start)?;

    // If any crate changed, confirm the changes with the user.
    prompt_user(changes, "crates", git_start).await
}

/// Check changes to npm packages since git_start to remind developers to
/// update changelogs and versions.
pub async fn check_npm(conf: &Conf, git_start: &str) -> Result<()> {
    if !conf.project.search_for_npm() {
        return Ok(());
    }
    println!("--> Looking for npm packages that changed since last release ...");

    // Find npm packages by looking for package.json files
    // then check each crate for git changes.
    let packages = files_by_name("package.json", conf.npm.ignored.as_slice()).await?;
    let changes = look_for_changes(&packages, git_start)?;

    // If any crate changed, confirm the changes with the user.
    prompt_user(changes, "npm packages", git_start).await
}

/// Recursively search for all files matching the given name.
///
/// The search is performed ignoring globs as described in `ignore::WalkBuilder`
/// as well as additional ignore globs specified in the project configuration.
async fn files_by_name(name: &'static str, ignores: &[String]) -> Result<Vec<PathBuf>> {
    let mut patterns = Vec::new();
    for ignore in ignores {
        let pattern = glob::Pattern::new(ignore)
            .with_context(|| format!("Invalid ignore glob pattern {}", ignore))?;
        patterns.push(pattern);
    }

    tokio::task::spawn_blocking(move || -> Result<_> {
        // Configure the iterator with the provided ignores.
        let mut walker = ignore::WalkBuilder::new(".");
        walker
            .sort_by_file_name(|a, b| a.cmp(b))
            .filter_entry(move |entry| {
                for pattern in &patterns {
                    let path = match entry.path().strip_prefix(".") {
                        Ok(path) => path,
                        Err(_) => entry.path(),
                    };
                    if pattern.matches_path(path) {
                        return false;
                    }
                }
                let is_dir = entry.path().is_dir();
                is_dir || entry.file_name() == name
            });

        // Look for files, ignoring directories, and collect their parents.
        let mut paths = Vec::new();
        for entry in walker.build() {
            // Ignore access errors and directories.
            let entry = match entry {
                Err(ignore::Error::Io(error)) if error.kind() == ErrorKind::PermissionDenied => {
                    continue
                }
                Ok(entry) if entry.path().is_dir() => continue,
                entry => entry.context(FILE_LIST_ERROR)?,
            };

            // Strip the leading . needed by ignore::Walk and "go up" to the parent.
            let entry = entry.into_path();
            let mut entry = match entry.strip_prefix(".") {
                Ok(entry) => entry.to_owned(),
                Err(_) => entry,
            };
            entry.pop();
            paths.push(entry);
        }
        Ok(paths)
    })
    .await
    .context(FILE_LIST_ERROR)?
}

/// Check git diff for changes to the given paths.
fn look_for_changes<'a>(paths: &'a [PathBuf], git_start: &str) -> Result<Vec<&'a PathBuf>> {
    let repo = Repository::discover(".").expect("unable to find repository");
    let start = repo
        .revparse_single(git_start)
        .context("Failed to parse starting git revision")?;
    let start = start
        .peel_to_tree()
        .context("Provided starting git revision does not point to a tree")?;

    let mut changes = Vec::new();
    for path in paths {
        let changed = super::git::changes_in_path(&repo, &start, &path)?;
        if changed {
            changes.push(path);
        }
    }
    Ok(changes)
}

/// If changes were found remind developers to update changelogs and versions.
async fn prompt_user(changes: Vec<&PathBuf>, name: &str, git_start: &str) -> Result<()> {
    if changes.is_empty() {
        eprintln!("Looks like nothing changed since {}", git_start);
        return Ok(());
    }
    println!(
        "It looks like the following {} were changed since {}:",
        name, git_start
    );
    for path in changes {
        let path = match path.display().to_string() {
            path if path == "" => String::from("(repository)"),
            path => path,
        };
        println!("  * {}", path);
    }
    let confirmed = tokio::task::spawn_blocking(|| {
        dialoguer::Confirm::new()
            .default(false)
            .with_prompt("Have all changelogs and versions been updated?")
            .interact()
    })
    .await
    .context(USER_INTERACT_ERROR)?
    .context(USER_INTERACT_ERROR)?;
    if confirmed {
        return Ok(());
    }
    ReleaseCheck::failed(anyhow::anyhow!(
        "You must confirm that all changelogs and versions were updated"
    ))
}

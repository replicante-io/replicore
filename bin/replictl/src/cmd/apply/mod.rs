//! Apply objects declarations to the Control Plane
use anyhow::Result;
use clap::Parser;
use futures::TryStreamExt;

mod load;
mod transform;

use crate::context::ContextStore;
use crate::Globals;

/// File glob for JSON files.
const JSON_GLOB: &str = "*.json";

/// File glob for YAML files.
const YAML_GLOB: &str = "*.yaml";

/// Alternative file glob for YAML files.
const YML_GLOB: &str = "*.yml";

/// Apply objects declarations to the Control Plane.
#[derive(Debug, Parser)]
pub struct ApplyCli {
    /// List of files to load manifests to apply from.
    pub file: Vec<String>,

    /// Expand paths to apply recursively instead of expanding only one level into directories.
    #[arg(short, long, default_value_t = false)]
    pub recursive: bool,

    /// Override scope (namespace, cluster and node) in manifests using context information.
    #[arg(long, default_value_t = false)]
    pub scope_override: bool,
}

/// Explore directories looking for files to apply.
fn glob_for_extension(
    files: &mut Vec<String>,
    path: &str,
    extension: &str,
    recurse: bool,
) -> Result<()> {
    let recursive = if recurse { "**" } else { "" };
    let pattern = format!("{}{}/{}", path, recursive, extension);
    for path in glob::glob(&pattern)? {
        let path = path?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("expanded path not valid unicode"))?
            .to_string();
        files.push(path);
    }
    Ok(())
}

/// Expand provided directories to supported files within them.
async fn process_directories(files: Vec<String>, recurse: bool) -> Result<Vec<String>> {
    let expanded = tokio::task::spawn_blocking(move || -> Result<_> {
        let mut expanded = Vec::new();
        for entry in files.into_iter() {
            // Check if the path points to a discovery.
            let path = std::path::Path::new(&entry);
            if !path.is_dir() {
                expanded.push(entry);
                continue;
            }

            // If so expand it to look for all files in it.
            glob_for_extension(&mut expanded, &entry, YAML_GLOB, recurse)?;
            glob_for_extension(&mut expanded, &entry, YML_GLOB, recurse)?;
            glob_for_extension(&mut expanded, &entry, JSON_GLOB, recurse)?;
        }
        Ok(expanded)
    })
    .await??;
    Ok(expanded)
}

/// Execute the selected `replictl apply` command.
pub async fn run(globals: &Globals, cmd: &ApplyCli) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    // If no files are given read from standard input.
    let files = cmd.file.clone();
    let mut files = process_directories(files, cmd.recursive).await?;
    if files.is_empty() {
        files.push("-".into());
    }

    // Load manifests and apply them one at a time.
    let manifests = self::load::manifests(files);
    futures::pin_mut!(manifests);
    while let Some(manifest) = manifests.try_next().await? {
        let manifest = match cmd.scope_override {
            false => manifest,
            true => self::transform::scope(globals, &context, manifest),
        };

        slog::debug!(globals.logger, "Applying manifest to control plane");
        client.apply(manifest).await?;
    }
    Ok(0)
}

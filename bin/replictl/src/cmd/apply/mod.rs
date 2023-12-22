//! Apply objects declarations to the Control Plane
use anyhow::Result;
use clap::Parser;
use futures::TryStreamExt;

mod load;
mod transform;

use crate::context::ContextStore;
use crate::Globals;

/// Apply objects declarations to the Control Plane.
#[derive(Debug, Parser)]
pub struct ApplyCli {
    /// List of files to load manifests to apply from.
    pub file: Vec<String>,

    /// Override scope (namespace, cluster and node) in manifests using context information.
    #[arg(long, default_value_t = false)]
    pub scope_override: bool,
}

/// Execute the selected `replictl apply` command.
pub async fn run(globals: &Globals, cmd: &ApplyCli) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    // If no files are given read from standard input.
    let mut files = cmd.file.clone();
    // TODO: support resolving directories into lists of files.
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

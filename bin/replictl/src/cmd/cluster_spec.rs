//! Inspect, delete or manipulate cluster specifications.
use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

use crate::context::ContextStore;
use crate::formatter::ops::ClusterSpecListOp;
use crate::Globals;

/// Inspect, delete or manipulate cluster specifications.
#[derive(Debug, Parser)]
pub struct ClusterSpecCli {
    /// Select the `replictl cluster` command to run.
    #[command(subcommand)]
    pub command: ClusterSpecCmd,
}

/// Possible platform commands to run.
#[derive(Debug, Subcommand)]
pub enum ClusterSpecCmd {
    /// Delete a cluster specification from the control plane.
    Delete,

    /// Lookup details for a cluster specification.
    Get,

    /// List cluster specifications on the control plane.
    List,

    /// Schedule a cluster orchestration task to execute in the background.
    Orchestrate,
}

/// Execute the selected `replictl platform` command.
pub async fn run(globals: &Globals, cmd: &ClusterSpecCli) -> Result<i32> {
    match &cmd.command {
        ClusterSpecCmd::Delete => delete(globals).await,
        ClusterSpecCmd::Get => get(globals).await,
        ClusterSpecCmd::List => list(globals).await,
        ClusterSpecCmd::Orchestrate => orchestrate(globals).await,
    }
}

async fn delete(globals: &Globals) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let name = context.cluster(&globals.cli.context)?;
    client.clusterspec(&ns_id, &name).delete().await?;
    println!("ClusterSpec '{name}' in namespace '{ns_id}' was deleted.");

    Ok(0)
}

async fn get(globals: &Globals) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let name = context.cluster(&globals.cli.context)?;
    let cluster_spec = client.clusterspec(&ns_id, &name).get().await?;
    globals.formatter.format(globals, cluster_spec);

    Ok(0)
}

async fn list(globals: &Globals) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let clusters = client.list().clusterspecs(&ns_id).await?;
    let mut formatter = globals.formatter.format(globals, ClusterSpecListOp);

    for cluster in clusters {
        formatter.append(&cluster)?;
    }

    formatter.finish()?;
    Ok(0)
}

async fn orchestrate(globals: &Globals) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let name = context.cluster(&globals.cli.context)?;
    client.clusterspec(&ns_id, &name).orchestrate().await?;

    println!("Orchestration of cluster '{name}' in namespace '{ns_id}' scheduled");
    Ok(0)
}

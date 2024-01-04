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

/// Lookup details for a cluster specification.
#[derive(Debug, Parser)]
pub struct ClusterSpecOpts {
    /// Name of the cluster specification to lookup.
    pub name: String,
}

/// Possible platform commands to run.
#[derive(Debug, Subcommand)]
pub enum ClusterSpecCmd {
    /// Delete a cluster specification from the control plane.
    Delete(ClusterSpecOpts),

    /// Lookup details for a cluster specification.
    Get(ClusterSpecOpts),

    /// List cluster specifications on the control plane.
    List,
}

/// Execute the selected `replictl platform` command.
pub async fn run(globals: &Globals, cmd: &ClusterSpecCli) -> Result<i32> {
    match &cmd.command {
        ClusterSpecCmd::Delete(cmd) => delete(globals, cmd).await,
        ClusterSpecCmd::Get(cmd) => get(globals, cmd).await,
        ClusterSpecCmd::List => list(globals).await,
    }
}

async fn delete(globals: &Globals, cmd: &ClusterSpecOpts) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let name = &cmd.name;
    client.clusterspec(&ns_id, name).delete().await?;
    println!("ClusterSpec '{name}' in namespace '{ns_id}' was deleted.");

    Ok(0)
}

async fn get(globals: &Globals, cmd: &ClusterSpecOpts) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let name = &cmd.name;
    let cluster_spec = client.clusterspec(&ns_id, name).get().await?;
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
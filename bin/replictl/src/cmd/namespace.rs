//! Inspect, delete or manipulate namespaces.
use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

use crate::context::ContextStore;
use crate::formatter::ops::NamespaceListOp;
use crate::Globals;

/// Inspect, delete or manipulate namespaces.
#[derive(Debug, Parser)]
pub struct NamespaceCli {
    /// Select the `replictl namespace` command to run.
    #[command(subcommand)]
    pub command: NamespaceCmd,
}

/// Possible namespace commands to run.
#[derive(Debug, Subcommand)]
pub enum NamespaceCmd {
    /// Delete a namespace (and all objects within it).
    Delete,

    /// Lookup details for a namespace.
    Get,

    /// List namespaces on the cluster.
    List,
}

/// Execute the selected `replictl namespace` command.
pub async fn run(globals: &Globals, cmd: &NamespaceCli) -> Result<i32> {
    match cmd.command {
        NamespaceCmd::Delete => delete(globals).await,
        NamespaceCmd::Get => get(globals).await,
        NamespaceCmd::List => list(globals).await,
    }
}

async fn delete(globals: &Globals) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    client.namespace(&ns_id).delete().await?;
    println!("Namespace '{ns_id}' and all resources in it set to delete asynchronously.");

    Ok(0)
}

async fn get(globals: &Globals) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let namespace = client.namespace(&ns_id).get().await?;
    globals.formatter.format(globals, namespace);

    Ok(0)
}

async fn list(globals: &Globals) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let namespaces = client.list().namespaces().await?;
    let mut formatter = globals.formatter.format(globals, NamespaceListOp);

    for namespace in namespaces {
        formatter.append(&namespace)?;
    }

    formatter.finish()?;
    Ok(0)
}

//! Inspect or manipulate node actions for a cluster.
use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

use crate::context::ContextStore;
use crate::formatter::ops::NActionListOp;
use crate::Globals;

/// Inspect or manipulate node actions for a cluster.
#[derive(Debug, Parser)]
pub struct NActionCli {
    /// Select the `replictl n-action` command to run.
    #[command(subcommand)]
    pub command: NActionCmd,
}

/// Possible node actions commands to run.
#[derive(Debug, Subcommand)]
pub enum NActionCmd {
    /// Approve a PENDING_APPROVE node action for scheduling.
    Approve(NActionGetOpts),

    /// Cancel a node action and prevent any further execution (including running actions).
    Cancel(NActionGetOpts),

    /// Lookup and display information about a node action.
    Get(NActionGetOpts),

    /// List node actions for the cluster.
    List(NActionListOpts),

    /// Reject a PENDING_APPROVE node action to prevent scheduling.
    Reject(NActionGetOpts),
}

/// Lookup and display information about a node action.
#[derive(Debug, Parser)]
pub struct NActionGetOpts {
    /// ID of the action to lookup.
    pub action_id: uuid::Uuid,
}

/// List node actions for the cluster.
#[derive(Debug, Parser)]
pub struct NActionListOpts {
    /// List node actions from all cluster nodes.
    #[arg(long, alias = "all-nodes", default_value_t = false)]
    pub all_nodes: bool,

    /// Exclude finished actions in the actions list.
    #[arg(long, alias = "no-all", default_value_t = false)]
    pub no_all: bool,
}

/// Execute the selected `replictl n-action` command.
pub async fn run(globals: &Globals, cmd: &NActionCli) -> Result<i32> {
    match cmd.command {
        NActionCmd::Approve(ref opts) => approve(globals, opts).await,
        NActionCmd::Cancel(ref opts) => cancel(globals, opts).await,
        NActionCmd::Get(ref opts) => get(globals, opts).await,
        NActionCmd::List(ref opts) => list(globals, opts).await,
        NActionCmd::Reject(ref opts) => reject(globals, opts).await,
    }
}

async fn approve(globals: &Globals, opts: &NActionGetOpts) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let cluster_id = context.cluster(&globals.cli.context)?;
    let node_id = context.node(&globals.cli.context)?;
    let action_id = opts.action_id;

    client
        .naction(&ns_id, &cluster_id, &node_id, action_id)
        .approve()
        .await?;
    println!("Node action approved for scheduling");
    Ok(0)
}

async fn cancel(globals: &Globals, opts: &NActionGetOpts) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let cluster_id = context.cluster(&globals.cli.context)?;
    let node_id = context.node(&globals.cli.context)?;
    let action_id = opts.action_id;

    client
        .naction(&ns_id, &cluster_id, &node_id, action_id)
        .cancel()
        .await?;
    println!("Node action cancelled and will not run any further");
    Ok(0)
}

async fn get(globals: &Globals, opts: &NActionGetOpts) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let cluster_id = context.cluster(&globals.cli.context)?;
    let node_id = context.node(&globals.cli.context)?;
    let action_id = opts.action_id;

    let action = client
        .naction(&ns_id, &cluster_id, &node_id, action_id)
        .get()
        .await?;
    globals.formatter.format(globals, action)?;

    Ok(0)
}

async fn list(globals: &Globals, opts: &NActionListOpts) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let cluster_id = context.cluster(&globals.cli.context)?;
    let node_id = match opts.all_nodes {
        true => None,
        false => context.node(&globals.cli.context).ok(),
    };
    let actions = client
        .list()
        .nactions(&ns_id, &cluster_id, &node_id, !opts.no_all)
        .await?;
    let mut formatter = globals.formatter.format(globals, NActionListOp);

    for action in actions {
        formatter.append(&action)?;
    }

    formatter.finish()?;
    Ok(0)
}

async fn reject(globals: &Globals, opts: &NActionGetOpts) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let cluster_id = context.cluster(&globals.cli.context)?;
    let node_id = context.node(&globals.cli.context)?;
    let action_id = opts.action_id;

    client
        .naction(&ns_id, &cluster_id, &node_id, action_id)
        .reject()
        .await?;
    println!("Node action rejected to prevent scheduling");
    Ok(0)
}

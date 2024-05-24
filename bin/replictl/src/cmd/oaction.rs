//! Inspect or manipulate orchestrator actions for a cluster.
use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

use crate::context::ContextStore;
use crate::formatter::ops::OActionListOp;
use crate::Globals;

/// Inspect or manipulate orchestrator actions for a cluster.
#[derive(Debug, Parser)]
pub struct OActionCli {
    /// Select the `replictl o-action` command to run.
    #[command(subcommand)]
    pub command: OActionCmd,
}

/// Possible orchestrator actions commands to run.
#[derive(Debug, Subcommand)]
pub enum OActionCmd {
    /// Approve a PENDING_APPROVE orchestrator action for scheduling.
    Approve(OActionGetOpts),

    /// Cancel an orchestrator action and prevent any further execution (including running actions).
    Cancel(OActionGetOpts),

    /// Lookup and display information about an orchestrator action.
    Get(OActionGetOpts),

    /// List orchestrator actions for the cluster.
    List(OActionListOpts),

    /// Reject a PENDING_APPROVE orchestrator action to prevent scheduling.
    Reject(OActionGetOpts),
}

/// Lookup and display information about an orchestrator action.
#[derive(Debug, Parser)]
pub struct OActionGetOpts {
    /// ID of the action to lookup.
    pub action_id: uuid::Uuid,
}

/// List orchestrator actions for the cluster.
#[derive(Debug, Parser)]
pub struct OActionListOpts {
    /// Exclude finished actions in the actions list.
    #[arg(long, alias = "no-all", default_value_t = false)]
    pub no_all: bool,
}

/// Execute the selected `replictl o-action` command.
pub async fn run(globals: &Globals, cmd: &OActionCli) -> Result<i32> {
    match cmd.command {
        OActionCmd::Approve(ref opts) => approve(globals, opts).await,
        OActionCmd::Cancel(ref opts) => cancel(globals, opts).await,
        OActionCmd::Get(ref opts) => get(globals, opts).await,
        OActionCmd::List(ref opts) => list(globals, opts).await,
        OActionCmd::Reject(ref opts) => reject(globals, opts).await,
    }
}

async fn approve(globals: &Globals, opts: &OActionGetOpts) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let cluster_id = context.cluster(&globals.cli.context)?;
    let action_id = opts.action_id;

    client.oaction(&ns_id, &cluster_id, action_id).approve().await?;
    println!("Orchestrator action approved for scheduling");
    Ok(0)
}

async fn cancel(globals: &Globals, opts: &OActionGetOpts) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let cluster_id = context.cluster(&globals.cli.context)?;
    let action_id = opts.action_id;

    client.oaction(&ns_id, &cluster_id, action_id).cancel().await?;
    println!("Orchestrator action cancelled and will not run any further");
    Ok(0)
}

async fn get(globals: &Globals, opts: &OActionGetOpts) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let cluster_id = context.cluster(&globals.cli.context)?;
    let action_id = opts.action_id;

    let action = client.oaction(&ns_id, &cluster_id, action_id).get().await?;
    globals.formatter.format(globals, action)?;

    Ok(0)
}

async fn list(globals: &Globals, opts: &OActionListOpts) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let cluster_id = context.cluster(&globals.cli.context)?;
    let actions = client
        .list()
        .oactions(&ns_id, &cluster_id, !opts.no_all)
        .await?;
    let mut formatter = globals.formatter.format(globals, OActionListOp);

    for action in actions {
        formatter.append(&action)?;
    }

    formatter.finish()?;
    Ok(0)
}

async fn reject(globals: &Globals, opts: &OActionGetOpts) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let cluster_id = context.cluster(&globals.cli.context)?;
    let action_id = opts.action_id;

    client.oaction(&ns_id, &cluster_id, action_id).reject().await?;
    println!("Orchestrator action rejected to prevent scheduling");
    Ok(0)
}

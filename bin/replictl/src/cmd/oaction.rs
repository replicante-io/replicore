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
    /// Lookup and display information about an orchestrator action.
    Get(OActionGetOpts),

    /// List orchestrator actions for the cluster.
    List(OActionListOpts),
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
    /// Include finished actions in the actions list.
    #[arg(long, default_value_t = false)]
    pub all: bool,
}

/// Execute the selected `replictl o-action` command.
pub async fn run(globals: &Globals, cmd: &OActionCli) -> Result<i32> {
    match cmd.command {
        OActionCmd::Get(ref opts) => get(globals, opts).await,
        OActionCmd::List(ref opts) => list(globals, opts).await,
    }
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
        .oactions(&ns_id, &cluster_id, opts.all)
        .await?;
    let mut formatter = globals.formatter.format(globals, OActionListOp);

    for action in actions {
        formatter.append(&action)?;
    }

    formatter.finish()?;
    Ok(0)
}

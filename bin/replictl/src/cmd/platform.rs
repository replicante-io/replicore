//! Inspect, delete or manipulate platforms.
use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

use crate::context::ContextStore;
use crate::formatter::ops::PlatformListOp;
use crate::Globals;

/// Inspect, delete or manipulate platforms.
#[derive(Debug, Parser)]
pub struct PlatformCli {
    /// Select the `replictl platform` command to run.
    #[command(subcommand)]
    pub command: PlatformCmd,
}

/// Lookup details for a platform.
#[derive(Debug, Parser)]
pub struct PlatformOpts {
    /// Name of the platform to lookup.
    pub name: String,
}

/// Possible platform commands to run.
#[derive(Debug, Subcommand)]
pub enum PlatformCmd {
    /// Delete a platform configuration from the control plane.
    Delete(PlatformOpts),

    /// Lookup details for a platform.
    Get(PlatformOpts),

    /// List platforms on the control plane.
    List,
}

/// Execute the selected `replictl platform` command.
pub async fn run(globals: &Globals, cmd: &PlatformCli) -> Result<i32> {
    match &cmd.command {
        PlatformCmd::Delete(cmd) => delete(globals, cmd).await,
        PlatformCmd::Get(cmd) => get(globals, cmd).await,
        PlatformCmd::List => list(globals).await,
    }
}

async fn delete(globals: &Globals, cmd: &PlatformOpts) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let name = &cmd.name;
    client.platform(&ns_id, name).delete().await?;
    println!("Platform '{name}' in namespace '{ns_id}' was deleted.");

    Ok(0)
}

async fn get(globals: &Globals, cmd: &PlatformOpts) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let name = &cmd.name;
    let platform = client.platform(&ns_id, name).get().await?;
    globals.formatter.format(globals, platform);

    Ok(0)
}

async fn list(globals: &Globals) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    let client = crate::client(&context)?;

    let ns_id = context.namespace(&globals.cli.context)?;
    let platforms = client.list().platforms(&ns_id).await?;
    let mut formatter = globals.formatter.format(globals, PlatformListOp);

    for platform in platforms {
        formatter.append(&platform)?;
    }

    formatter.finish()?;
    Ok(0)
}

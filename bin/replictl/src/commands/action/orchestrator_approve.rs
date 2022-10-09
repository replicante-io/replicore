use anyhow::Result;
use slog::Logger;

use super::CommonOpt;
use crate::apiclient::RepliClient;
use crate::context::ContextStore;
use crate::Cli;

/// Execute the command.
pub async fn execute(logger: &Logger, cli: &Cli, approve_opt: &CommonOpt) -> Result<i32> {
    let context = ContextStore::active_context(logger, cli).await?;
    let _ns = context.namespace(&cli.context)?;
    let cluster = context.cluster(&cli.context)?;
    let action = approve_opt.action;
    let client = RepliClient::new(logger, context).await?;
    client.action_orchestrator_approve(&cluster, action).await?;
    println!("Orchestrator action approved for scheduling");
    Ok(0)
}

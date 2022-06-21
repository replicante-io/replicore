use anyhow::Result;
use slog::Logger;

use super::CommonOpt;
use crate::apiclient::RepliClient;
use crate::context::ContextStore;
use crate::Opt;

/// Execute the command.
pub async fn execute(logger: &Logger, opt: &Opt, approve_opt: &CommonOpt) -> Result<i32> {
    let context = ContextStore::active_context(logger, opt).await?;
    let _ns = context.namespace(&opt.context)?;
    let cluster = context.cluster(&opt.context)?;
    let action = approve_opt.action;
    let client = RepliClient::new(logger, context).await?;
    client.action_orchestrator_approve(&cluster, action).await?;
    println!("Orchestrator action approved for scheduling");
    Ok(0)
}

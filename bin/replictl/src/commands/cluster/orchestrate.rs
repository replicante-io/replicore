use anyhow::Result;
use slog::Logger;

use crate::apiclient::RepliClient;
use crate::context::ContextStore;

/// Execute the selected command.
pub async fn execute(logger: &Logger, cli: &crate::Cli) -> Result<i32> {
    let context = ContextStore::active_context(logger, cli).await?;
    let _ns = context.namespace(&cli.context)?;
    let cluster = context.cluster(&cli.context)?;
    let client = RepliClient::new(logger, context).await?;
    client.orchestrate_cluster(&cluster).await?;
    println!("Cluster orchestration scheduled");
    Ok(0)
}

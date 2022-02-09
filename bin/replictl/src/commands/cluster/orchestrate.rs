use anyhow::Result;
use slog::Logger;

use crate::apiclient::RepliClient;
use crate::context::ContextStore;

/// Execute the selected command.
pub async fn execute(logger: &Logger, opt: &crate::Opt) -> Result<i32> {
    let context = ContextStore::active_context(logger, opt).await?;
    let _ns = context.namespace(&opt.context)?;
    let cluster = context.cluster(&opt.context)?;
    let client = RepliClient::new(logger, context).await?;
    client.orchestrate_cluster(&cluster).await?;
    println!("Cluster orchestration scheduled");
    Ok(0)
}

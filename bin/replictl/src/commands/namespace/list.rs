use anyhow::Result;
use slog::Logger;

use crate::apiclient::RepliClient;
use crate::context::ContextStore;

/// Execute the selected command.
pub async fn execute(logger: &Logger, cli: &crate::Cli) -> Result<i32> {
    let context = ContextStore::active_context(logger, cli).await?;
    let client = RepliClient::new(logger, context).await?;
    println!("Available namespaces:");
    for namespace in client.namespace_list().await? {
        println!("{}", namespace.ns_id);
    }
    Ok(0)
}

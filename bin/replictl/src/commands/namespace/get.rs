use anyhow::Result;
use slog::Logger;

use crate::apiclient::RepliClient;
use crate::context::ContextStore;

/// Execute the selected command.
pub async fn execute(logger: &Logger, cli: &crate::Cli) -> Result<i32> {
    let context = ContextStore::active_context(logger, cli).await?;
    let ns = context.namespace(&cli.context)?;
    let client = RepliClient::new(logger, context).await?;
    let namespace = client.namespace_get(&ns).await?;
    let namespace =
        serde_json::to_string_pretty(&namespace).expect("failed json encoding of namespace");
    println!("{}", namespace);
    Ok(0)
}

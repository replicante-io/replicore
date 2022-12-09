use anyhow::Result;
use slog::Logger;

use crate::apiclient::RepliClient;
use crate::context::ContextStore;

/// Execute the selected command.
pub async fn execute(logger: &Logger, cli: &crate::Cli, opt: &super::CommonOpt) -> Result<i32> {
    let context = ContextStore::active_context(logger, cli).await?;
    let ns = context.namespace(&cli.context)?;
    let client = RepliClient::new(logger, context).await?;
    let platform = client.platform_get(&ns, &opt.platform).await?;
    let platform =
        serde_json::to_string_pretty(&platform).expect("failed json encoding of platform");
    println!("{}", platform);
    Ok(0)
}

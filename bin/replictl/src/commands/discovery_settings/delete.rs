use anyhow::Result;
use slog::Logger;

use crate::apiclient::RepliClient;
use crate::context::ContextStore;

/// Execute the selected command.
pub async fn execute(
    logger: &Logger,
    cli: &crate::Cli,
    delete_opt: &super::CommonOpt,
) -> Result<i32> {
    let context = ContextStore::active_context(logger, cli).await?;
    let ns = context.namespace(&cli.context)?;
    let name = &delete_opt.discovery_name;
    let client = RepliClient::new(logger, context).await?;
    client.discovery_settings_delete(&ns, name).await?;
    println!("Deleted DiscoverySettings object {}.{}", ns, name);
    Ok(0)
}

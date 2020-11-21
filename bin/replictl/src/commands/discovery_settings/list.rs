use anyhow::Result;
use slog::Logger;

use crate::apiclient::RepliClient;
use crate::context::ContextStore;

/// Execute the selected command.
pub async fn execute(logger: &Logger, opt: &crate::Opt) -> Result<i32> {
    let context = ContextStore::active_context(logger, opt).await?;
    let ns = context.namespace(&opt.context)?;
    let client = RepliClient::new(logger, context).await?;
    let settings = client.discovery_settings_list(&ns).await?;
    println!(
        "Listing names of DiscoverySettings objects in the {} namespace:",
        ns
    );
    for name in settings {
        println!("{}", name);
    }
    Ok(0)
}

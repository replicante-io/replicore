use anyhow::Context;
use anyhow::Result;
use slog::Logger;

use crate::context::ContextStore;
use crate::errors::ContextNotFound;
use crate::Cli;

/// Execute the command.
pub async fn execute(logger: &Logger, cli: &Cli) -> Result<i32> {
    let store = ContextStore::load(logger, cli).await?;
    let name = store.active_context_name(cli);
    let context = store.get(&name);

    // Print an error if the context does not exist.
    let context = match context {
        Some(context) => context,
        None => anyhow::bail!(ContextNotFound::for_name(name)),
    };

    // Format and print the context.
    let encoded = serde_yaml::to_string(&context)
        .with_context(|| format!("failed to YAML encode context {}", name))?;
    tokio::task::spawn_blocking(move || println!("{}", encoded))
        .await
        .with_context(|| format!("failed to output described context '{}'", name))?;
    Ok(0)
}

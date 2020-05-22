use anyhow::Context;
use anyhow::Result;
use slog::Logger;

use crate::context::ContextNotFound;
use crate::context::ContextStore;
use crate::Opt;

/// Execute the command.
pub async fn execute(logger: &Logger, opt: &Opt) -> Result<i32> {
    let store = ContextStore::load(logger, opt).await?;
    let name = store.active_context_name(opt);
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
        .with_context(|| format!("failed to output descibed context '{}'", name))?;
    Ok(0)
}

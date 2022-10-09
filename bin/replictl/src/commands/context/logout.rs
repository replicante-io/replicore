use anyhow::Result;
use slog::Logger;

use crate::context::ContextStore;
use crate::errors::ContextNotFound;
use crate::Cli;

/// Execute the command.
pub async fn execute(logger: &Logger, cli: &Cli) -> Result<i32> {
    let mut store = ContextStore::load(logger, cli).await?;
    let name = store.active_context_name(cli);
    if store.get(&name).is_none() {
        anyhow::bail!(ContextNotFound::for_name(name));
    }
    store.remove(name);
    store.save(logger, cli).await?;
    Ok(0)
}

use anyhow::Result;
use slog::Logger;

use crate::context::ContextNotFound;
use crate::context::ContextStore;
use crate::Opt;

/// Execute the command.
pub async fn execute(logger: &Logger, opt: &Opt) -> Result<i32> {
    let mut store = ContextStore::load(logger, opt).await?;
    let name = store.active_context_name(opt);
    if store.get(&name).is_none() {
        anyhow::bail!(ContextNotFound::for_name(name));
    }
    store.remove(name);
    store.save(logger, opt).await?;
    Ok(0)
}

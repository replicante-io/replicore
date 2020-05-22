use anyhow::Context;
use anyhow::Result;
use dialoguer::Select;
use slog::Logger;

use crate::context::ContextStore;
use crate::Opt;

const INTERACT_ERROR: &str = "error while interacting with the user";

/// Execute the command.
pub async fn execute(logger: &Logger, opt: &Opt) -> Result<i32> {
    let mut store = ContextStore::load(logger, opt).await?;
    let active_name = store.active_context_name(opt);
    let mut contexts: Vec<_> = store.iter().map(|(name, _)| name.to_string()).collect();
    contexts.sort();
    let default = contexts
        .iter()
        .enumerate()
        .find(|(_, name)| name.as_str() == active_name)
        .map(|(index, _)| index);

    // Interact with the user to select a context.
    let name = tokio::task::spawn_blocking(move || -> Result<_> {
        let mut select = Select::new();
        select
            .with_prompt("Select default context (esc or q to clear selection)")
            .items(&contexts);
        if let Some(default) = default {
            select.default(default);
        }

        let index = select.interact_opt()?;
        Ok(index.map(|index| contexts[index].clone()))
    })
    .await
    .context(INTERACT_ERROR)?
    .context(INTERACT_ERROR)?;

    // Update the active context name and persistthe store to disk.
    store.set_active_context_name(name);
    store.save(logger, opt).await?;
    Ok(0)
}

//! Select the active context, the one used when none are specified.
use anyhow::Result;
use inquire::Select;

use crate::context::ContextStore;
use crate::Globals;

/// Select the active context, the one used when none are specified.
pub async fn run(globals: &Globals) -> Result<i32> {
    let store = ContextStore::load(globals).await?;
    let current = store.active_id(globals);
    let options: Vec<_> = store.iter().map(|(name, _)| name.to_string()).collect();
    let selected = options.iter().position(|name| name == current);

    let selection = tokio::task::spawn_blocking(move || {
        let mut prompt = Select::new(
            "Select the active context for use in future commands",
            options,
        );
        if let Some(index) = selected {
            prompt = prompt.with_starting_cursor(index);
        }
        prompt.prompt()
    })
    .await??;
    let selection = match selection {
        selection if selection.is_empty() => None,
        selection => Some(selection),
    };

    let mut store = store;
    store.set_active_id(selection);
    store.save(globals).await?;
    Ok(0)
}

use anyhow::Context;
use anyhow::Result;
use dialoguer::Input;
use slog::Logger;

use crate::context::ContextStore;
use crate::errors::ContextNotFound;
use crate::Cli;

const INTERACT_ERROR: &str = "error while interacting with the user";

/// Execute the command.
pub async fn execute(logger: &Logger, cli: &Cli) -> Result<i32> {
    let mut store = ContextStore::load(logger, cli).await?;
    let name = store.active_context_name(cli);
    let context = store.get(&name);

    // Print an error if the context does not exist.
    let mut context = match context {
        Some(context) => context,
        None => anyhow::bail!(ContextNotFound::for_name(name)),
    };

    // Use the CLI options as default if set.
    let namespace = context.namespace(&cli.context).ok();
    let cluster = context.cluster(&cli.context).ok();
    let node = context.node(&cli.context).ok();

    // Interact with the user to update the scope.
    context.scope.namespace =
        input_optional("Select a namespace (empty to clear selection)", &namespace).await?;
    context.scope.cluster =
        input_optional("Select a cluster (empty to clear selection)", &cluster).await?;
    context.scope.node = input_optional("Select a node (empty to clear selection)", &node).await?;

    // Save the updated context to the store and the store to disk.
    store.upsert(name, context);
    store.save(logger, cli).await?;
    Ok(0)
}

/// Ask the user to provide an optional path.
async fn input_optional(prompt: &str, initial: &Option<String>) -> Result<Option<String>> {
    let initial = initial.as_deref().unwrap_or("").to_string();
    let prompt = prompt.to_string();
    let value: String = tokio::task::spawn_blocking(move || {
        Input::new()
            .with_prompt(prompt)
            .with_initial_text(initial)
            .allow_empty(true)
            .interact()
    })
    .await
    .context(INTERACT_ERROR)?
    .context(INTERACT_ERROR)?;
    match value {
        path if path.is_empty() => Ok(None),
        path => Ok(Some(path)),
    }
}

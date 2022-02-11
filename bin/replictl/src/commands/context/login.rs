use anyhow::Context as _;
use anyhow::Result;
use dialoguer::Input;
use slog::debug;
use slog::Logger;

use crate::context::Connection;
use crate::context::Context;
use crate::context::ContextStore;
use crate::Opt;

const INTERACT_ERROR: &str = "error while interacting with the user";

/// Execute the command.
pub async fn execute(logger: &Logger, opt: &Opt) -> Result<i32> {
    let mut store = ContextStore::load(logger, opt).await?;
    let name = store.active_context_name(opt);
    let mut context = match store.get(&name) {
        Some(context) => context,
        None => {
            debug!(
                logger, "Context not found, creating a new one";
                "context" => &name,
            );
            let connection = Connection {
                ca_bundle: None,
                client_key: None,
                url: String::from(""),
            };
            let scope = Default::default();
            Context { connection, scope }
        }
    };

    // Interact with the user to create/update the connection.
    let url = context.connection.url.clone();
    context.connection.url = tokio::task::spawn_blocking(|| {
        Input::new()
            .with_prompt("Replicante API address")
            .with_initial_text(url)
            .interact()
    })
    .await
    .context(INTERACT_ERROR)?
    .context(INTERACT_ERROR)?;
    context.connection.ca_bundle = input_optional_path(
        "(Optional) API CA certificate file",
        &context.connection.ca_bundle,
    )
    .await?;
    context.connection.client_key = input_optional_path(
        "(Optional) API client certificate bundle",
        &context.connection.client_key,
    )
    .await?;

    // Save the updated context to the store and the store to disk.
    store.upsert(name, context);
    store.save(logger, opt).await?;
    Ok(0)
}

/// Ask the user to provide an optional path.
async fn input_optional_path(prompt: &str, initial: &Option<String>) -> Result<Option<String>> {
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
        path if path.starts_with('/') => Ok(Some(path)),
        path => {
            let current_dir = std::env::current_dir().expect("the current directory to be set");
            let current_dir = current_dir
                .as_path()
                .to_str()
                .expect("the current directory to be a valid path");
            let path = format!("{}/{}", current_dir, path);
            Ok(Some(path))
        }
    }
}

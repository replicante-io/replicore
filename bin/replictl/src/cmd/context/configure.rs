//! Configure or update the connection options for a RepliCore server.
use anyhow::Result;
use inquire::validator::Validation;
use inquire::Text;

use crate::context::Context;
use crate::context::ContextStore;
use crate::Globals;

/// Connect to a RepliCore server or update connection options.
pub async fn run(globals: &Globals) -> Result<i32> {
    let store = ContextStore::load(globals).await?;
    let active = store.active_id(globals);
    println!("Configuring Replicante Control Plane access for context {active}");

    // Lookup the context to edit or create an empty one.
    let context = store.get(active).unwrap_or_else(create_empty_context);
    let context = tokio::task::spawn_blocking(move || -> Result<Context> {
        let mut context = context;
        context.connection.url = Text::new("Replicante Control Plane URL:")
            .with_initial_value(&context.connection.url)
            .with_placeholder("Required")
            .with_validator(|value: &str| match value.is_empty() {
                false => Ok(Validation::Valid),
                true => Ok(Validation::Invalid(
                    "Control Plane URL cannot be empty".into(),
                )),
            })
            .prompt()?;

        let ca_bundle = Text::new("PEM Bundle of Certificate Authorities to trust:")
            .with_initial_value(context.connection.ca_bundle.as_deref().unwrap_or(""))
            .with_placeholder("Optional")
            .prompt()?;
        let ca_bundle = match ca_bundle {
            bundle if bundle.is_empty() => None,
            bundle => Some(bundle),
        };
        context.connection.ca_bundle = ca_bundle;
        let client_key = Text::new("Client Certificate's Private Key to send the Control Plane:")
            .with_initial_value(context.connection.client_key.as_deref().unwrap_or(""))
            .with_placeholder("Optional")
            .prompt()?;
        let client_key = match client_key {
            key if key.is_empty() => None,
            key => Some(key),
        };
        context.connection.client_key = client_key;

        Ok(context)
    })
    .await??;

    // Update the contexts store and save it to disk.
    let active = active.to_string();
    let mut store = store;
    store.upsert(active, context);
    store.save(globals).await?;
    Ok(0)
}

/// Create an empty context to use as a placeholder when a new context is needed.
fn create_empty_context() -> Context {
    let connection = crate::context::Connection {
        ca_bundle: None,
        client_key: None,
        url: String::from(""),
    };
    Context {
        connection,
        scope: Default::default(),
    }
}

//! Change scope attributes, such as namespace or cluster, of a context.
use anyhow::Result;
use inquire::Text;

use crate::context::ContextStore;
use crate::Globals;

/// Change scope attributes of a context.
pub async fn run(globals: &Globals) -> Result<i32> {
    let store = ContextStore::load(globals).await?;
    let context = store.get_active(globals)?;

    // Lookup the current values as defaults (including CLI overrides).
    let namespace = context.namespace(&globals.cli.context).ok();
    let cluster = context.cluster(&globals.cli.context).ok();
    let node = context.node(&globals.cli.context).ok();

    // Prompt the user for updates.
    let namespace = Text::new("Implicit namespace to use:")
        .with_initial_value(namespace.as_deref().unwrap_or(""))
        .with_placeholder("None selected")
        .prompt()?;
    let cluster = Text::new("Implicit cluster to use:")
        .with_initial_value(cluster.as_deref().unwrap_or(""))
        .with_placeholder("None selected")
        .prompt()?;
    let node = Text::new("Implicit node to use:")
        .with_initial_value(node.as_deref().unwrap_or(""))
        .with_placeholder("None selected")
        .prompt()?;

    let mut context = context;
    context.scope.namespace = match namespace {
        namespace if namespace.is_empty() => None,
        namespace => Some(namespace),
    };
    context.scope.cluster = match cluster {
        cluster if cluster.is_empty() => None,
        cluster => Some(cluster),
    };
    context.scope.node = match node {
        node if node.is_empty() => None,
        node => Some(node),
    };

    // Save the changes.
    let name = store.active_id(globals).to_owned();
    let mut store = store;
    store.upsert(name, context);
    store.save(globals).await?;
    Ok(0)
}

//! Ensure the selected context configuration is removed from.
use anyhow::Result;
use inquire::Confirm;

use crate::context::ContextStore;
use crate::Globals;

/// Change scope attributes of a context.
pub async fn run(globals: &Globals) -> Result<i32> {
    let mut store = ContextStore::load(globals).await?;
    let active = store.active_id(globals).to_owned();
    let confirm = Confirm::new(&format!("Deleting context {active}, can't be undone"))
        .with_default(false)
        .prompt()?;

    if confirm {
        store.remove(&active);
        store.save(globals).await?;
        println!("Context {active} was deleted")
    }
    Ok(0)
}

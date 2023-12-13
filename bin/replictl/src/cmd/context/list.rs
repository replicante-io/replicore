//! List known RepliCore servers in the client config store.
use anyhow::Result;

use crate::context::ContextStore;
use crate::formatter::ops::ContextListOp;
use crate::Globals;

/// List configured contexts.
pub async fn run(globals: &Globals) -> Result<i32> {
    let store = ContextStore::load(globals).await?;
    let mut formatter = globals.formatter.format(globals, ContextListOp);

    let active = store.active_id(globals);
    for (name, context) in store.iter() {
        formatter.append(name, context, active == name)?;
    }

    formatter.finish()?;
    Ok(0)
}

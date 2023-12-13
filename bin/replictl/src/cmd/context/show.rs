//! List known RepliCore servers in the client config store.
use anyhow::Result;

use crate::context::ContextStore;
use crate::Globals;

/// Show details about the current `replictl` context.
pub async fn run(globals: &Globals) -> Result<i32> {
    let context = ContextStore::active(globals).await?;
    globals.formatter.format(globals, context);
    Ok(0)
}

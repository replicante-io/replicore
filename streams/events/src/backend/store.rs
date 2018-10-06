use std::sync::Arc;
use slog::Logger;

use replicante_data_models::Event;
use replicante_data_store::Store;

use super::super::Result;
use super::StreamInterface;


/// Wrap the store interface.
pub struct StoreInterface {
    store: Store,
}

impl StoreInterface {
    pub fn new(logger: Logger, store: Store) -> Arc<StreamInterface> {
        debug!(logger, "Using store backend for events stream");
        let store = StoreInterface { store };
        Arc::new(store)
    }
}

impl StreamInterface for StoreInterface {
    fn emit(&self, event: Event) -> Result<()> {
        self.store.persist_event(event)?;
        Ok(())
    }
}

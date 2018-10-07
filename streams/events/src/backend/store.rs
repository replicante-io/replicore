use std::sync::Arc;
use slog::Logger;

use replicante_data_models::Event;

use replicante_data_store::EventsFilters;
use replicante_data_store::EventsOptions;
use replicante_data_store::Store;

use super::super::Iter;
use super::super::Result;
use super::super::ScanFilters;
use super::super::ScanOptions;
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

    fn scan(&self, _filters: ScanFilters, options: ScanOptions) -> Result<Iter> {
        let filters = EventsFilters::all();
        let options = into_store_options(options);
        let iter = self.store.events(filters, options)?;
        let iter = iter.map(|e| Ok(e?));
        Ok(Iter::new(iter))
    }
}


fn into_store_options(options: ScanOptions) -> EventsOptions {
    let mut opts = EventsOptions::default();
    opts.limit = options.limit;
    opts.reverse = options.reverse;
    opts
}

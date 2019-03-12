use std::sync::Arc;

use failure::ResultExt;
use slog::Logger;

use replicante_data_models::Event;
use replicante_data_store::EventsFilters;
use replicante_data_store::EventsOptions;
use replicante_data_store::Store;

use super::super::Error;
use super::super::ErrorKind;
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
        self.store.persist_event(event)
            .with_context(|_| ErrorKind::StoreWrite("event")).map_err(Error::from)
    }

    fn scan(&self, filters: ScanFilters, options: ScanOptions) -> Result<Iter> {
        let filters = into_store_filters(filters);
        let options = into_store_options(options);
        let iter = self.store.events(filters, options)
            .with_context(|_| ErrorKind::StoreRead("events"))?;
        let iter = iter.map(|event| {
            event.with_context(|_| ErrorKind::StoreRead("event")).map_err(Error::from)
        });
        Ok(Iter::new(iter))
    }
}


fn into_store_filters(filters: ScanFilters) -> EventsFilters {
    let mut fils = EventsFilters::default();
    fils.cluster_id = filters.cluster_id;
    fils.event = filters.event;
    fils.exclude_snapshots = filters.exclude_snapshots;
    fils.exclude_system_events = filters.exclude_system_events;
    fils.start_from = filters.start_from;
    fils.stop_at = filters.stop_at;
    fils
}

fn into_store_options(options: ScanOptions) -> EventsOptions {
    let mut opts = EventsOptions::default();
    opts.limit = options.limit;
    opts.reverse = options.reverse;
    opts
}

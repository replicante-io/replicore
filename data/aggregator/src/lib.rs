extern crate prometheus;
//#[macro_use]
extern crate slog;

extern crate replicante_coordinator;
extern crate replicante_data_models;
extern crate replicante_data_store;

use slog::Logger;

use replicante_coordinator::NonBlockingLockWatcher;
use replicante_data_store::Store;

mod metrics;

pub use self::metrics::register_metrics;

/// Node (agent and datastore) status aggregator logic.
pub struct Aggregator {
    _logger: Logger,
    _store: Store,
}

impl Aggregator {
    pub fn new(_logger: Logger, _store: Store) -> Aggregator {
        Aggregator { _logger, _store }
    }

    pub fn process(&self, _cluster_id: String, _lock: NonBlockingLockWatcher) {
        // TODO(stefano): implement cluster aggregation.
    }
}

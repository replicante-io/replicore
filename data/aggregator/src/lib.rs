extern crate prometheus;
//#[macro_use]
extern crate slog;

extern crate replicante_coordinator;
extern crate replicante_data_models;
extern crate replicante_data_store;

use prometheus::Registry;
use slog::Logger;

use replicante_coordinator::NonBlockingLockWatcher;
use replicante_data_models::ClusterDiscovery;
use replicante_data_store::Store;


/// Node (agent and datastore) status aggregator logic.
pub struct Aggregator {
    //logger: Logger,
    //store: Store,
}

impl Aggregator {
    pub fn new(_logger: Logger, _store: Store) -> Aggregator {
        Aggregator {
            //logger,
            //store,
        }
    }

    pub fn register_metrics(_logger: &Logger, _registry: &Registry) {
        // TODO(stefano): implement metrics registration.
    }

    pub fn process(&self, _cluster: ClusterDiscovery, _lock: NonBlockingLockWatcher) {
        // TODO(stefano): implement cluster aggregation.
    }
}

extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate prometheus;
#[macro_use]
extern crate slog;

extern crate replicante_coordinator;
extern crate replicante_data_models;
extern crate replicante_data_store;

use failure::ResultExt;
use slog::Logger;

use replicante_coordinator::NonBlockingLockWatcher;
use replicante_data_models::ClusterDiscovery;
use replicante_data_store::store::Store;

mod cluster_meta;
mod error;
mod metrics;

use self::cluster_meta::ClusterMetaAggregator;
use self::metrics::AGGREGATE_DURATION;
use self::metrics::AGGREGATE_ERRORS_COUNT;

pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::metrics::register_metrics;

/// Node (agent and datastore) status aggregator logic.
pub struct Aggregator {
    logger: Logger,
    store: Store,
}

impl Aggregator {
    pub fn new(logger: Logger, store: Store) -> Aggregator {
        Aggregator { logger, store }
    }

    /// Process aggregations for a cluster.
    ///
    /// If the aggregation process fails due to core-related issues (store errors,
    /// internal logic, ...) the process is aborted and the error propagated.
    pub fn aggregate(
        &self,
        discovery: ClusterDiscovery,
        lock: NonBlockingLockWatcher,
    ) -> Result<()> {
        let _timer = AGGREGATE_DURATION.start_timer();
        self.inner_process(discovery, lock).map_err(|error| {
            AGGREGATE_ERRORS_COUNT.inc();
            error
        })
    }
}

impl Aggregator {
    /// Wrapped logic to handle error cases only once.
    pub fn inner_process(
        &self,
        discovery: ClusterDiscovery,
        lock: NonBlockingLockWatcher,
    ) -> Result<()> {
        let cluster_id = discovery.cluster_id.clone();
        debug!(self.logger, "Aggregating cluster"; "cluster_id" => &cluster_id);

        // (Re-)Aggregate cluster meta.
        let mut meta = ClusterMetaAggregator::new(&discovery);
        meta.aggregate(self.store.clone())?;
        let meta = meta.generate();
        if !lock.inspect() {
            return Err(ErrorKind::ClusterLockLost(cluster_id).into());
        }
        self.store
            .legacy()
            .persist_cluster_meta(meta)
            .with_context(|_| ErrorKind::StoreWrite("ClusterMeta"))?;

        // Generated all aggrgations.
        Ok(())
    }
}

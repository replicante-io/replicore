use failure::ResultExt;
use opentracingrust::Log;
use opentracingrust::Span;
use slog::debug;
use slog::Logger;

use replicante_models_core::cluster::ClusterDiscovery;
use replicante_service_coordinator::NonBlockingLockWatcher;
use replicante_store_primary::store::Store;
use replicante_util_tracing::fail_span;

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
        span: &mut Span,
    ) -> Result<()> {
        span.log(Log::new().log("stage", "aggregate"));
        let _timer = AGGREGATE_DURATION.start_timer();
        self.inner_process(discovery, lock, span).map_err(|error| {
            AGGREGATE_ERRORS_COUNT.inc();
            fail_span(error, span)
        })
    }
}

impl Aggregator {
    /// Wrapped logic to handle error cases only once.
    pub fn inner_process(
        &self,
        discovery: ClusterDiscovery,
        lock: NonBlockingLockWatcher,
        span: &mut Span,
    ) -> Result<()> {
        let cluster_id = discovery.cluster_id.clone();
        debug!(self.logger, "Aggregating cluster"; "cluster_id" => &cluster_id);

        // (Re-)Aggregate cluster meta.
        let mut meta = ClusterMetaAggregator::new(&discovery);
        meta.aggregate(self.store.clone(), span)?;
        let meta = meta.generate();
        if !lock.inspect() {
            return Err(ErrorKind::ClusterLockLost(cluster_id).into());
        }
        self.store
            .legacy()
            .persist_cluster_meta(meta, span.context().clone())
            .with_context(|_| ErrorKind::StoreWrite("ClusterMeta"))?;

        // Generated all aggrgations.
        Ok(())
    }
}

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
    pub fn process(&self, discovery: ClusterDiscovery, lock: NonBlockingLockWatcher) -> Result<()> {
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
        // Initialise cluster aggregators.
        let mut meta = ClusterMetaAggregator::new(self.store.clone(), lock);
        let cluster_id = discovery.cluster_id.clone();
        debug!(self.logger, "Aggregating cluster"; "cluster_id" => &cluster_id);

        // Visit the discovery.
        meta.visit_discovery(&discovery)?;

        // Visit agents.
        let agents = self
            .store
            .agents(cluster_id.clone())
            .iter()
            .with_context(|_| ErrorKind::StoreRead("cluster agents"))?;
        for agent in agents {
            let agent = agent.with_context(|_| ErrorKind::StoreRead("agent"))?;
            meta.visit_agent(&agent)?;
        }

        // Visit agents info.
        let agents_info = self
            .store
            .agents(cluster_id.clone())
            .iter_info()
            .with_context(|_| ErrorKind::StoreRead("cluster agents info"))?;
        for agent in agents_info {
            let agent = agent.with_context(|_| ErrorKind::StoreRead("agent info"))?;
            meta.visit_agent_info(&agent)?;
        }

        // Visit node records.
        let nodes = self
            .store
            .nodes(cluster_id.clone())
            .iter()
            .with_context(|_| ErrorKind::StoreRead("cluster nodes"))?;
        for node in nodes {
            let node = node.with_context(|_| ErrorKind::StoreRead("node"))?;
            meta.visit_node(&node)?;
        }

        // Visit shards.
        let shards = self
            .store
            .shards(cluster_id.clone())
            .iter()
            .with_context(|_| ErrorKind::StoreRead("cluster shards"))?;
        for shard in shards {
            let shard = shard.with_context(|_| ErrorKind::StoreRead("shard"))?;
            meta.visit_shard(&shard)?;
        }

        // Commit generated metadata.
        meta.commit()?;
        Ok(())
    }
}

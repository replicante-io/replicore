extern crate failure;
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
use replicante_data_store::Store;

mod cluster_meta;
mod error;
mod metrics;

use self::cluster_meta::ClusterMetaAggregator;

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
    pub fn process(
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
        let agents = self.store.cluster_agents(cluster_id.clone())
            .with_context(|_| ErrorKind::StoreRead("cluster agents"))?;
        for agent in agents {
            let agent = agent.with_context(|_| ErrorKind::StoreRead("agent"))?;
            meta.visit_agent(&agent)?;
        }

        // Visit agents info.
        let agents_info = self.store.cluster_agents_info(cluster_id.clone())
            .with_context(|_| ErrorKind::StoreRead("cluster agents info"))?;
        for agent in agents_info {
            let agent = agent.with_context(|_| ErrorKind::StoreRead("agent info"))?;
            meta.visit_agent_info(&agent)?;
        }

        // Visit node records.
        let nodes = self.store.cluster_nodes(cluster_id.clone())
            .with_context(|_| ErrorKind::StoreRead("cluster nodes"))?;
        for node in nodes {
            let node = node.with_context(|_| ErrorKind::StoreRead("node"))?;
            meta.visit_node(&node)?;
        }

        // Visit shards.
        let shards = self.store.cluster_shards(cluster_id.clone())
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

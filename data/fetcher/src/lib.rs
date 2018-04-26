#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

extern crate prometheus;
#[macro_use]
extern crate slog;

extern crate replicante_agent_client;
extern crate replicante_data_models;
extern crate replicante_data_store;

use error_chain::ChainedError;
use prometheus::Registry;
use slog::Logger;

use replicante_agent_client::HttpClient;
use replicante_data_models::Agent;
use replicante_data_models::AgentStatus;
use replicante_data_models::ClusterDiscovery;
use replicante_data_store::Store;


mod agent;
mod errors;
mod meta;
mod metrics;
mod node;
mod shard;

use self::agent::AgentFetcher;
use self::meta::ClusterMetaBuilder;
use self::meta::MetaFetcher;

use self::metrics::FETCHER_ERRORS_COUNT;
use self::metrics::register_metrics;

use self::node::NodeFetcher;
use self::shard::ShardFetcher;

pub use self::errors::Error;
pub use self::errors::ErrorKind;
pub use self::errors::ResultExt;
pub use self::errors::Result;


/// Node (agent and datastore) status fetching and processing logic.
///
/// The Fetcher is responsible for:
///
///   1. For each node:
///     1. Attempt to fetch agent info.
///     2. Persist `AgentInfo` record (if fetch succeeded).
///     3. Attempt to fetch node info (if agent is up).
///     4. Persist `Node` record (if fetch succeeded).
///     5. Attempt to fetch shards status (only if agent and datastore are up).
///     6. Persist each `Shard` record (if fetch succeeded).
///     7. Persist the `Agent` record.
///   2. Generate and persist `ClusterMeta` record.
pub struct Fetcher {
    agent: AgentFetcher,
    logger: Logger,
    meta: MetaFetcher,
    node: NodeFetcher,
    shard: ShardFetcher,
}

impl Fetcher {
    pub fn new(logger: Logger, store: Store) -> Fetcher {
        let agent = AgentFetcher::new(logger.clone(), store.clone());
        let meta = MetaFetcher::new(logger.clone(), store.clone());
        let node = NodeFetcher::new(logger.clone(), store.clone());
        let shard = ShardFetcher::new(logger.clone(), store);
        Fetcher {
            agent,
            logger,
            meta,
            node,
            shard,
        }
    }

    pub fn register_metrics(logger: &Logger, registry: &Registry) {
        register_metrics(logger, registry)
    }

    pub fn process(&self, cluster: ClusterDiscovery) {
        let name = cluster.name.clone();
        let mut meta = ClusterMetaBuilder::new(cluster.name);
        for node in cluster.nodes {
            let result = self.process_target(name.clone(), node.clone(), &mut meta);
            if let Err(error) = result {
                FETCHER_ERRORS_COUNT.with_label_values(&[&name]).inc();
                let error = error.display_chain().to_string();
                error!(
                    self.logger, "Failed to process cluster node";
                    "cluster" => name.clone(), "node" => node,
                    "error" => error
                );
            }
        }
        self.meta.persist_meta(meta.build());
    }
}

impl Fetcher {
    fn process_target(
        &self, cluster: String, node: String, meta: &mut ClusterMetaBuilder
    ) -> Result<()> {
        meta.node_inc();
        let client = HttpClient::new(node.clone())?;
        let mut agent = Agent::new(cluster.clone(), node.clone(), AgentStatus::Up);

        let result = self.agent.process_agent(&client, cluster.clone(), node.clone());
        if let Err(error) = result {
            let message = error.display_chain().to_string();
            agent.status = AgentStatus::AgentDown(message);
            self.agent.persist_agent(agent);
            return Err(error);
        };

        let result = self.node.process_node(&client, meta);
        if let Err(error) = result {
            let message = error.display_chain().to_string();
            agent.status = AgentStatus::DatastoreDown(message);
            self.agent.persist_agent(agent);
            return Err(error);
        };

        let result = self.shard.process_shards(&client, cluster, node);
        if let Err(error) = result {
            let message = error.display_chain().to_string();
            agent.status = AgentStatus::DatastoreDown(message);
            self.agent.persist_agent(agent);
            return Err(error);
        };

        self.agent.persist_agent(agent);
        Ok(())
    }
}

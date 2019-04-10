extern crate failure;
extern crate failure_derive;

#[macro_use]
extern crate lazy_static;
extern crate prometheus;
#[macro_use]
extern crate slog;

extern crate replicante_agent_client;
extern crate replicante_coordinator;
extern crate replicante_data_models;
extern crate replicante_data_store;
extern crate replicante_streams_events;
extern crate replicante_util_failure;

use std::time::Duration;

use failure::ResultExt;
use prometheus::Registry;
use slog::Logger;

use replicante_agent_client::HttpClient;
use replicante_coordinator::NonBlockingLockWatcher;
use replicante_data_models::Agent;
use replicante_data_models::AgentStatus;
use replicante_data_models::ClusterDiscovery;
use replicante_data_store::Store;
use replicante_streams_events::EventsStream;
use replicante_util_failure::failure_info;
use replicante_util_failure::format_fail;


mod agent;
mod error;
mod meta;
mod metrics;
mod node;
mod shard;
mod snapshotter;

use self::agent::AgentFetcher;
use self::meta::ClusterMetaBuilder;
use self::meta::MetaFetcher;

use self::metrics::FETCHER_ERRORS_COUNT;
use self::metrics::register_metrics;

use self::node::NodeFetcher;
use self::shard::ShardFetcher;

pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::snapshotter::Snapshotter;


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
    timeout: Duration,
}

impl Fetcher {
    pub fn new(logger: Logger, events: EventsStream, store: Store, timeout: Duration) -> Fetcher {
        let agent = AgentFetcher::new(events.clone(), store.clone());
        let meta = MetaFetcher::new(store.clone());
        let node = NodeFetcher::new(events.clone(), store.clone());
        let shard = ShardFetcher::new(events, store);
        Fetcher {
            agent,
            logger,
            meta,
            node,
            shard,
            timeout,
        }
    }

    pub fn register_metrics(logger: &Logger, registry: &Registry) {
        register_metrics(logger, registry)
    }

    pub fn process(&self, cluster: ClusterDiscovery, lock: NonBlockingLockWatcher) {
        let name = cluster.cluster.clone();
        let mut meta = ClusterMetaBuilder::new(cluster.cluster);

        for node in cluster.nodes {
            // Exit early if lock was lost.
            if !lock.inspect() {
                warn!(
                    self.logger, "Cluster fetcher lock lost, skipping futher nodes";
                    "cluster" => &name
                );
                return;
            }

            let result = self.process_target(&name, &node, &mut meta);
            if let Err(error) = result {
                FETCHER_ERRORS_COUNT.with_label_values(&[&name]).inc();
                error!(
                    self.logger, "Failed to process cluster node";
                    "cluster" => &name, "node" => node, failure_info(&error)
                );
            }
        }

        if let Err(error) = self.meta.persist_meta(meta.build()) {
            FETCHER_ERRORS_COUNT.with_label_values(&[&name]).inc();
            error!(
                self.logger, "Failed to persist cluster metadata";
                "cluster" => name, failure_info(&error)
            );
        }
    }
}

impl Fetcher {
    fn process_target(
        &self, cluster: &str, node: &str, meta: &mut ClusterMetaBuilder
    ) -> Result<()> {
        meta.node_inc();
        let client = HttpClient::make(node.to_string(), self.timeout.clone())
            .with_context(|_| ErrorKind::AgentConnect(node.to_string()))?;
        let mut agent = Agent::new(cluster.to_string(), node.to_string(), AgentStatus::Up);

        let result = self.agent.process_agent_info(&client, cluster.to_string(), node.to_string());
        if let Err(error) = result {
            let message = format_fail(&error);
            agent.status = AgentStatus::AgentDown(message);
            self.agent.process_agent(agent)?;
            return Err(error);
        };

        let result = self.node.process_node(&client, meta);
        if let Err(error) = result {
            let message = format_fail(&error);
            agent.status = AgentStatus::NodeDown(message);
            self.agent.process_agent(agent)?;
            return Err(error);
        };

        let result = self.shard.process_shards(&client, cluster, node);
        if let Err(error) = result {
            let message = format_fail(&error);
            agent.status = AgentStatus::NodeDown(message);
            self.agent.process_agent(agent)?;
            return Err(error);
        };

        self.agent.process_agent(agent)
    }
}

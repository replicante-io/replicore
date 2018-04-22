use std::collections::HashSet;

use error_chain::ChainedError;
use slog::Logger;

use replicante_agent_client::Client;
use replicante_agent_client::HttpClient;

use replicante_data_models::Agent;
use replicante_data_models::AgentInfo;
use replicante_data_models::AgentStatus;
use replicante_data_models::ClusterDiscovery;
use replicante_data_models::ClusterMeta;
use replicante_data_models::Node;
use replicante_data_models::Shard;

use replicante_data_store::Store;

use super::metrics::DISCOVERY_PROCESS_ERRORS_COUNT;
use super::super::Result;


struct ClusterMetaBuilder {
    cluster: String,
    kinds: HashSet<String>,
    nodes: i32,
}

impl ClusterMetaBuilder {
    pub fn build(self) -> ClusterMeta {
        let mut meta = ClusterMeta::new(self.cluster, "<OVERRIDE>", self.nodes);
        meta.kinds = self.kinds.into_iter().collect();
        meta
    }

    pub fn new(cluster: String) -> ClusterMetaBuilder {
        ClusterMetaBuilder {
            cluster,
            kinds: HashSet::new(),
            nodes: 0,
        }
    }

    pub fn node_inc(&mut self) {
        self.nodes += 1;
    }

    pub fn node_kind(&mut self, kind: String) {
        self.kinds.insert(kind);
    }
}


/// Node (agent and datastore) state fetching and processing logic.
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
    logger: Logger,
    store: Store,
}

impl Fetcher {
    pub fn new(logger: Logger, store: Store) -> Fetcher {
        Fetcher {
            logger,
            store,
        }
    }

    pub fn process(&self, cluster: ClusterDiscovery) {
        let name = cluster.name.clone();
        let mut meta = ClusterMetaBuilder::new(cluster.name);
        for node in cluster.nodes {
            let result = self.process_target(name.clone(), node.clone(), &mut meta);
            match result {
                Ok(_) => (),
                Err(error) => {
                    DISCOVERY_PROCESS_ERRORS_COUNT.inc();
                    let error = error.display_chain().to_string();
                    error!(
                        self.logger, "Failed to process cluster node";
                        "cluster" => name.clone(), "node" => node,
                        "error" => error
                    );
                }
            }
        }
        self.persist_meta(meta.build());
    }
}

impl Fetcher {
    fn persist_agent(&self, agent: Agent) {
        let cluster = agent.cluster.clone();
        let host = agent.host.clone();
        let old = match self.store.agent(cluster.clone(), host.clone()) {
            Ok(old) => old,
            Err(error) => {
                DISCOVERY_PROCESS_ERRORS_COUNT.inc();
                let error = error.display_chain().to_string();
                error!(
                    self.logger, "Failed to fetch agent";
                    "cluster" => cluster, "host" => host, "error" => error
                );
                return;
            }
        };

        // TODO: Emit agent events.

        if old != Some(agent.clone()) {
            match self.store.persist_agent(agent) {
                Ok(_) => (),
                Err(error) => {
                    DISCOVERY_PROCESS_ERRORS_COUNT.inc();
                    let error = error.display_chain().to_string();
                    error!(
                        self.logger, "Failed to persist agent";
                        "cluster" => cluster, "host" => host, "error" => error
                    );
                }
            };
        }
    }

    fn persist_agent_info(&self, agent: AgentInfo) {
        let cluster = agent.cluster.clone();
        let host = agent.host.clone();
        let old = match self.store.agent_info(cluster.clone(), host.clone()) {
            Ok(old) => old,
            Err(error) => {
                DISCOVERY_PROCESS_ERRORS_COUNT.inc();
                let error = error.display_chain().to_string();
                error!(
                    self.logger, "Failed to fetch agent info";
                    "cluster" => cluster, "host" => host, "error" => error
                );
                return;
            }
        };

        // TODO: Emit agent events.

        if old != Some(agent.clone()) {
            match self.store.persist_agent_info(agent) {
                Ok(_) => (),
                Err(error) => {
                    DISCOVERY_PROCESS_ERRORS_COUNT.inc();
                    let error = error.display_chain().to_string();
                    error!(
                        self.logger, "Failed to persist agent info";
                        "cluster" => cluster, "host" => host, "error" => error
                    );
                }
            };
        }
    }

    fn persist_meta(&self, meta: ClusterMeta) {
        let name = meta.name.clone();
        match self.store.persist_cluster_meta(meta) {
            Ok(_) => (),
            Err(error) => {
                DISCOVERY_PROCESS_ERRORS_COUNT.inc();
                let error = error.display_chain().to_string();
                error!(
                    self.logger, "Failed to persist cluster metadata";
                    "cluster" => name, "error" => error
                );
            }
        };
    }

    fn persist_node(&self, node: Node) {
        let cluster = node.cluster.clone();
        let name = node.name.clone();
        let old = match self.store.node(cluster.clone(), name.clone()) {
            Ok(old) => old,
            Err(error) => {
                DISCOVERY_PROCESS_ERRORS_COUNT.inc();
                let error = error.display_chain().to_string();
                error!(
                    self.logger, "Failed to fetch node info";
                    "cluster" => cluster, "name" => name, "error" => error
                );
                return;
            }
        };

        // TODO: Emit node events.

        if old != Some(node.clone()) {
            match self.store.persist_node(node) {
                Ok(_) => (),
                Err(error) => {
                    DISCOVERY_PROCESS_ERRORS_COUNT.inc();
                    let error = error.display_chain().to_string();
                    error!(
                        self.logger, "Failed to persist node info";
                        "cluster" => cluster, "name" => name, "error" => error
                    );
                }
            };
        }
    }

    fn persist_shard(&self, shard: Shard) {
        let cluster = shard.cluster.clone();
        let node = shard.node.clone();
        let id = shard.id.clone();
        let old = match self.store.shard(cluster.clone(), node.clone(), id.clone()) {
            Ok(old) => old,
            Err(error) => {
                DISCOVERY_PROCESS_ERRORS_COUNT.inc();
                let error = error.display_chain().to_string();
                error!(
                    self.logger, "Failed to fetch shard info";
                    "cluster" => cluster, "node" => node, "id" => id,
                    "error" => error
                );
                return;
            }
        };

        // TODO: Emit shard events.

        if old != Some(shard.clone()) {
            match self.store.persist_shard(shard) {
                Ok(_) => (),
                Err(error) => {
                    DISCOVERY_PROCESS_ERRORS_COUNT.inc();
                    let error = error.display_chain().to_string();
                    error!(
                        self.logger, "Failed to persist node info";
                        "cluster" => cluster, "node" => node, "id" => id,
                        "error" => error
                    );
                }
            };
        }
    }

    fn process_agent(&self, client: &Client, cluster: String, node: String) -> Result<()> {
        let info = client.info()?;
        let info = AgentInfo::new(cluster, node, info.agent);
        self.persist_agent_info(info);
        Ok(())
    }

    fn process_node(&self, client: &Client, meta: &mut ClusterMetaBuilder) -> Result<()> {
        let info = client.info()?;
        let node = Node::new(info.datastore);
        meta.node_kind(node.kind.clone());
        self.persist_node(node);
        Ok(())
    }

    fn process_shards(&self, client: &Client, cluster: String, node: String) -> Result<()> {
        let status = client.status()?;
        for shard in status.shards {
            let shard = Shard::new(cluster.clone(), node.clone(), shard);
            self.persist_shard(shard);
        }
        Ok(())
    }

    fn process_target(
        &self, cluster: String, node: String, meta: &mut ClusterMetaBuilder
    ) -> Result<()> {
        meta.node_inc();
        let client = HttpClient::new(node.clone())?;
        let mut agent = Agent::new(cluster.clone(), node.clone(), AgentStatus::Up);
        match self.process_agent(&client, cluster.clone(), node.clone()) {
            Ok(_) => (),
            Err(error) => {
                let message = error.display_chain().to_string();
                agent.status = AgentStatus::AgentDown(message);
                self.persist_agent(agent);
                return Err(error);
            }
        };
        match self.process_node(&client, meta) {
            Ok(_) => (),
            Err(error) => {
                let message = error.display_chain().to_string();
                agent.status = AgentStatus::DatastoreDown(message);
                self.persist_agent(agent);
                return Err(error);
            }
        };
        match self.process_shards(&client, cluster, node) {
            Ok(_) => (),
            Err(error) => {
                let message = error.display_chain().to_string();
                agent.status = AgentStatus::DatastoreDown(message);
                self.persist_agent(agent);
                return Err(error);
            }
        };
        self.persist_agent(agent);
        Ok(())
    }
}

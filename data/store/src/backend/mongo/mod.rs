use mongodb::Client;
use mongodb::ThreadedClient;

use prometheus::Registry;
use slog::Logger;

use replicante_data_models::Agent;
use replicante_data_models::AgentInfo;
use replicante_data_models::ClusterDiscovery;
use replicante_data_models::ClusterMeta;
use replicante_data_models::Node;
use replicante_data_models::Shard;

use super::super::InnerStore;
use super::super::Result;
use super::super::ResultExt;
use super::super::config::MongoDBConfig;


mod constants;
mod metrics;

mod agent;
mod cluster;
mod datastore;

use self::constants::FAIL_CLIENT;
use self::metrics::register_metrics;

use self::agent::AgentStore;
use self::datastore::DatastoreStore;
use self::cluster::ClusterStore;


/// MongoDB-backed storage layer.
///
/// # Special collection requirements
///
///   * `events`: capped collection or TTL indexed.
///
/// # Expected indexes
///
///   * Index on `clusters_meta`: `(nodes: -1, name: 1)`
///   * Unique index on `agents`: `(cluster: 1, host: 1)`
///   * Unique index on `agents_info`: `(cluster: 1, host: 1)`
///   * Unique index on `clusters_meta`: `name: 1`
///   * Unique index on `discoveries`: `name: 1`
///   * Unique index on `nodes`: `(cluster: 1, name: 1)`
///   * Unique index on `shards`: `(cluster: 1, name: 1, id: 1)`
pub struct MongoStore {
    agents: AgentStore,
    clusters: ClusterStore,
    datastores: DatastoreStore,
}

impl InnerStore for MongoStore {
    fn agent(&self, cluster: String, host: String) -> Result<Option<Agent>> {
        self.agents.agent(cluster, host)
    }

    fn agent_info(&self, cluster: String, host: String) -> Result<Option<AgentInfo>> {
        self.agents.agent_info(cluster, host)
    }

    fn cluster_discovery(&self, cluster: String) -> Result<Option<ClusterDiscovery>> {
        self.clusters.cluster_discovery(cluster)
    }

    fn cluster_meta(&self, cluster: String) -> Result<Option<ClusterMeta>> {
        self.clusters.cluster_meta(cluster)
    }

    fn find_clusters(&self, search: String, limit: u8) -> Result<Vec<ClusterMeta>> {
        self.clusters.find_clusters(search, limit)
    }

    fn node(&self, cluster: String, name: String) -> Result<Option<Node>> {
        self.datastores.node(cluster, name)
    }

    fn persist_agent(&self, agent: Agent) -> Result<()> {
        self.agents.persist_agent(agent)
    }

    fn persist_agent_info(&self, agent: AgentInfo) -> Result<()> {
        self.agents.persist_agent_info(agent)
    }

    fn persist_cluster_meta(&self, meta: ClusterMeta) -> Result<()> {
        self.clusters.persist_cluster_meta(meta)
    }

    fn persist_discovery(&self, cluster: ClusterDiscovery) -> Result<()> {
        self.clusters.persist_discovery(cluster)
    }

    fn persist_node(&self, node: Node) -> Result<()> {
        self.datastores.persist_node(node)
    }

    fn persist_shard(&self, shard: Shard) -> Result<()> {
        self.datastores.persist_shard(shard)
    }

    fn shard(&self, cluster: String, node: String, id: String) -> Result<Option<Shard>> {
        self.datastores.shard(cluster, node, id)
    }

    fn top_clusters(&self) -> Result<Vec<ClusterMeta>> {
        self.clusters.top_clusters()
    }
}

impl MongoStore {
    /// Creates a mongodb-backed store.
    pub fn new(config: MongoDBConfig, logger: Logger, registry: &Registry) -> Result<MongoStore> {
        info!(logger, "Configuring MongoDB as storage layer");
        let db = config.db.clone();
        let client = Client::with_uri(&config.uri).chain_err(|| FAIL_CLIENT)?;
        let agents = AgentStore::new(client.clone(), db.clone());
        let datastores = DatastoreStore::new(client.clone(), db.clone());
        let clusters = ClusterStore::new(client.clone(), db.clone());

        register_metrics(&logger, registry);
        Ok(MongoStore {
            agents,
            datastores,
            clusters,
        })
    }
}

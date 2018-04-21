use mongodb::Client;
use mongodb::ThreadedClient;

use prometheus::Registry;
use slog::Logger;

use replicante_data_models::ClusterDiscovery;
use replicante_data_models::ClusterMeta;
use replicante_data_models::Node;

use super::super::InnerStore;
use super::super::Result;
use super::super::ResultExt;
use super::super::config::MongoDBConfig;


mod clusters;
mod constants;
mod metrics;
mod datastore;

use self::constants::FAIL_CLIENT;
use self::metrics::register_metrics;

use self::clusters::ClusterStore;
use self::datastore::NodeStore;


/// MongoDB-backed storage layer.
///
/// # Special collection requirements
///
///   * `events`: capped collection or TTL indexed.
///
/// # Expected indexes
///
///   * Index on `cluster_meta`: `(name: 1, nodes: -1)`
///   * Unique index on `agents`: `(cluster: 1, host: 1)`
///   * Unique index on `agents_info`: `(cluster: 1, host: 1)`
///   * Unique index on `cluster_meta`: `name: 1`
///   * Unique index on `dicoveries`: `name: 1`
///   * Unique index on `nodes`: `(cluster: 1, name: 1)`
///   * Unique index on `shards`: `(cluster: 1, name: 1, shard: 1)`
pub struct MongoStore {
    clusters: ClusterStore,
    nodes: NodeStore,
}

impl InnerStore for MongoStore {
    fn cluster_discovery(&self, cluster: String) -> Result<ClusterDiscovery> {
        self.clusters.cluster_discovery(cluster)
    }

    fn cluster_meta(&self, cluster: String) -> Result<ClusterMeta> {
        self.clusters.cluster_meta(cluster)
    }

    fn find_clusters(&self, search: String, limit: u8) -> Result<Vec<ClusterMeta>> {
        self.clusters.find_clusters(search, limit)
    }

    fn top_clusters(&self) -> Result<Vec<ClusterMeta>> {
        self.clusters.top_clusters()
    }

    fn persist_cluster_meta(&self, meta: ClusterMeta) -> Result<Option<ClusterMeta>> {
        self.clusters.persist_cluster_meta(meta)
    }

    fn persist_discovery(&self, cluster: ClusterDiscovery) -> Result<Option<ClusterDiscovery>> {
        self.clusters.persist_discovery(cluster)
    }

    fn persist_node(&self, node: Node) -> Result<Option<Node>> {
        self.nodes.persist_node(node)
    }
}

impl MongoStore {
    /// Creates a mongodb-backed store.
    pub fn new(config: MongoDBConfig, logger: Logger, registry: &Registry) -> Result<MongoStore> {
        info!(logger, "Configuring MongoDB as storage layer");
        let db = config.db.clone();
        let client = Client::with_uri(&config.uri).chain_err(|| FAIL_CLIENT)?;
        let clusters = ClusterStore::new(client.clone(), db.clone());
        let nodes = NodeStore::new(client.clone(), db.clone());

        register_metrics(&logger, registry);
        Ok(MongoStore {
            clusters,
            nodes,
        })
    }
}

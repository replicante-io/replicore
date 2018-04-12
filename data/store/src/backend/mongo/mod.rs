use mongodb::Client;
use mongodb::ThreadedClient;

use prometheus::Registry;
use slog::Logger;

use replicante_data_models::Cluster;
use replicante_data_models::Node;
use replicante_data_models::webui::ClusterListItem;

use super::super::InnerStore;
use super::super::Result;
use super::super::ResultExt;
use super::super::config::MongoDBConfig;


mod clusters;
mod constants;
mod metrics;
mod nodes;

use self::constants::FAIL_CLIENT;
use self::metrics::register_metrics;

use self::clusters::ClusterStore;
use self::nodes::NodeStore;


/// MongoDB-backed storage layer.
///
/// # Special collection requirements
///
///   * `events`: capped collection or TTL indexed.
///
/// # Expected indexes
///
///   * Index on `cluster_lists`: `(name: 1, nodes: -1)`
///   * Unique index on `cluster_lists`: `name: 1`
///   * Unique index on `clusters`: `name: 1`
///   * Unique index on `nodes`: `(info.agent.cluster: 1, info.agent.name: 1)`
pub struct MongoStore {
    clusters: ClusterStore,
    nodes: NodeStore,
}

impl InnerStore for MongoStore {
    fn find_clusters(&self, search: String, limit: u8) -> Result<Vec<ClusterListItem>> {
        self.clusters.find_clusters(search, limit)
    }

    fn fetch_top_clusters(&self) -> Result<Vec<ClusterListItem>> {
        self.clusters.fetch_top_clusters()
    }

    fn persist_cluster(&self, cluster: Cluster) -> Result<Option<Cluster>> {
        self.clusters.persist_cluster(cluster)
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

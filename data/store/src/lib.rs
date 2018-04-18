#[macro_use]
extern crate bson;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate mongodb;

extern crate prometheus;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate slog;

#[cfg(test)]
extern crate replicante_agent_models;
extern crate replicante_data_models;


use std::sync::Arc;

use prometheus::Registry;
use slog::Logger;

use replicante_data_models::Cluster;
use replicante_data_models::Node;

use replicante_data_models::webui::ClusterMeta;


mod backend;
mod config;
mod errors;

// Cargo builds dependencies in debug mode instead of test mode.
// That means that `cfg(test)` cannot be used if the mock is used outside the crate.
#[cfg(debug_assertions)]
pub mod mock;

pub use self::config::Config;
pub use self::errors::*;

use self::backend::mongo::MongoStore;


/// Public interface to the persistent storage layer.
///
/// This interface abstracts every interaction with the persistence layer and
/// hides implementation details about storage software and data encoding.
///
/// # Overview
/// Different data types (models) are owned by a component and different data units
/// (model instances) are owned by a running replicante component instance.
///
/// Owned means that only one component's running instance can generate or update the data.
/// This avoids most issues with concurrent data updates (it will never be possible to prevent
/// datastores from changing while data is collected from agents).
///
/// Ownership of data also simplifies operations that require a "full view" of the cluster
/// because one conponent's instance will be uniquely responsible for creating this "full view".
///
/// # Data flow
///
///   1. The discovery component (only one active in the entire cluster):
///     1. Discovers clusters with all their nodes.
///     2. Detects new clusters and nodes as well as nodes leaving the cluster.
///   2. The datafetch components (as many as desired, at least one):
///     1. Take exclusive ownership of each cluster.
///     2. Periodically fetch the state of each agent.
///     3. Build cluster metadata documents (incrementally, while iterating over agents).
///     4. Update agents, nodes, and shards models with the new data.
///     5. Compare known state with the newly fetched state to generate events.
#[derive(Clone)]
pub struct Store(Arc<InnerStore>);

impl Store {
    /// Instantiate a new storage interface.
    pub fn new(config: Config, logger: Logger, registry: &Registry) -> Result<Store> {
        let store = match config {
            Config::MongoDB(config) => Arc::new(MongoStore::new(config, logger, registry)?),
        };
        Ok(Store(store))
    }

    /// Fetches discovery information about a cluster.
    ///
    /// If the cluster is not found an error is returned.
    pub fn cluster_discovery<S>(&self, cluster: S) -> Result<Cluster>
        where S: Into<String>,
    {
        self.0.cluster_discovery(cluster.into())
    }

    /// Fetches metadata about a cluster.
    ///
    /// If the cluster is not found an error is returned.
    pub fn cluster_meta<S>(&self, cluster: S) -> Result<ClusterMeta>
        where S: Into<String>,
    {
        self.0.cluster_meta(cluster.into())
    }

    /// Searches for a list of clusters with names matching the search term.
    ///
    /// A limited number of cluster is returned to avoid abuse.
    /// To find more clusters refine the search (paging is not supported).
    pub fn find_clusters<S>(&self, search: S, limit: u8) -> Result<Vec<ClusterMeta>>
        where S: Into<String>,
    {
        self.0.find_clusters(search.into(), limit)
    }

    /// Fetches overvew details of the top clusters.
    ///
    /// Clusters are sorted by number of nodes in the cluster.
    pub fn top_clusters(&self) -> Result<Vec<ClusterMeta>> {
        self.0.top_clusters()
    }

    /// Persists information about a cluster.
    ///
    /// If the cluster is known it will be updated and the old model is returned.
    /// Tf the cluster is new it will be created and `None` will be returned.
    ///
    /// Clusters are uniquely identified by their name.
    pub fn persist_cluster(&self, cluster: Cluster) -> Result<Option<Cluster>> {
        self.0.persist_cluster(cluster)
    }

    /// Persists information about a node.
    ///
    /// If the node is known it will be updated and the old model is returned.
    /// Tf the node is new it will be created and `None` will be returned.
    ///
    /// Nodes are uniquely identified by `(cluster, name)`.
    pub fn persist_node(&self, node: Node) -> Result<Option<Node>> {
        self.0.persist_node(node)
    }

    /// Instantiate a `Store` that wraps the given `MockStore`.
    // Cargo builds dependencies in debug mode instead of test mode.
    // That means that `cfg(test)` cannot be used if the mock is used outside the crate.
    #[cfg(debug_assertions)]
    pub fn mock(inner: Arc<self::mock::MockStore>) -> Store {
        Store(inner)
    }
}


/// Private interface to the persistence storage layer.
///
/// Allows multiple possible datastores to be used as well as mocks for testing.
trait InnerStore: Send + Sync {
    /// See `Store::cluster_discovery` for details.
    fn cluster_discovery(&self, cluster: String) -> Result<Cluster>;

    /// See `Store::cluster_meta` for details.
    fn cluster_meta(&self, cluster: String) -> Result<ClusterMeta>;

    /// See `Store::find_clusters` for details.
    fn find_clusters(&self, search: String, limit: u8) -> Result<Vec<ClusterMeta>>;

    /// See `Store::top_clusters` for details.
    fn top_clusters(&self) -> Result<Vec<ClusterMeta>>;

    /// See `Store::persist_cluster` for details.
    fn persist_cluster(&self, cluster: Cluster) -> Result<Option<Cluster>>;

    /// See `Store::persist_node` for details.
    fn persist_node(&self, node: Node) -> Result<Option<Node>>;
}

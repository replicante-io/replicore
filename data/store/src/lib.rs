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

use replicante_data_models::webui::ClusterListItem;


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
/// hides implementation details about storage software and data layout.
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

    /// Searches for a list of clusters with names matching the search term.
    ///
    /// A limited number of cluster is returned to avoid abuse.
    /// To find more clusters refine the search (paging is not supported).
    pub fn find_clusters<S>(&self, search: S, limit: u8) -> Result<Vec<String>>
        where S: Into<String>,
    {
        self.0.find_clusters(search.into(), limit)
    }

    /// Fetches overvew details of the top clusters.
    ///
    /// Clusters are sorted by number of nodes in the cluster.
    pub fn fetch_top_clusters(&self) -> Result<Vec<ClusterListItem>> {
        self.0.fetch_top_clusters()
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
    /// See `Store::find_clusters` for details.
    fn find_clusters(&self, search: String, limit: u8) -> Result<Vec<String>>;

    /// See `Store::fetch_top_clusters` for details.
    fn fetch_top_clusters(&self) -> Result<Vec<ClusterListItem>>;

    /// See `Store::persist_cluster` for details.
    fn persist_cluster(&self, cluster: Cluster) -> Result<Option<Cluster>>;

    /// See `Store::persist_node` for details.
    fn persist_node(&self, node: Node) -> Result<Option<Node>>;
}

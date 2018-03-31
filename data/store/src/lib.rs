#[macro_use]
extern crate bson;
#[macro_use]
extern crate error_chain;
extern crate mongodb;

extern crate serde;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate replicante_agent_models;
extern crate replicante_data_models;


use std::sync::Arc;

use replicante_data_models::Node;


mod backend;
mod config;
mod errors;

#[cfg(test)]
pub mod mock;

pub use self::config::Config;
pub use self::errors::*;

use self::backend::mongo::MongoStore;


/// Public interface to the persistent storage layer.
///
/// This interface abstracts every interaction with the persistence layer and
/// hides implementation details about storage software and data layout.
pub struct Store(Arc<InnerStore>);

impl Store {
    /// Instantiate a new storage interface.
    pub fn new(config: Config) -> Result<Store> {
        let store = match config {
            Config::MongoDB(config) => Arc::new(MongoStore::new(config)?),
        };
        Ok(Store(store))
    }

    /// Persists information about a node.
    ///
    /// If the node is known it will be updated, if it is new it will be created.
    /// Nodes are uniquely identified by `(cluster, name)`.
    ///
    /// If the node was found the return value is a `Some` with the old node,
    /// otherwise a `None` is returned.
    pub fn persist_node(&self, node: Node) -> Result<Option<Node>> {
        self.0.persist_node(node)
    }

    /// Instantiate a `Store` that wraps the given `MockStore`.
    #[cfg(test)]
    pub fn mock(inner: Arc<self::mock::MockStore>) -> Store {
        Store(inner)
    }
}


/// Private interface to the persistence storage layer.
///
/// Allows multiple possible datastores to be used as well as mocks for testing.
trait InnerStore: Send + Sync {
    /// See `Store::persist_node` for details.
    fn persist_node(&self, node: Node) -> Result<Option<Node>>;
}

#[macro_use]
extern crate bson;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate mongodb;

extern crate prometheus;
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

use replicante_data_models::Node;


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
    /// See `Store::persist_node` for details.
    fn persist_node(&self, node: Node) -> Result<Option<Node>>;
}

use std::sync::Arc;

use slog::Logger;

use super::BackendConfig;
use super::Config;
use super::NodeId;
use super::Result;
use super::Tombstone;
use super::backend;
use super::backend::Backend;


/// Interface to access distributed coordination services.
#[derive(Clone)]
pub struct Coordinator(Arc<Backend>);

impl Coordinator {
    pub fn new(config: Config, logger: Logger) -> Result<Coordinator> {
        let node_id = {
            let mut node = NodeId::new();
            node.extra(config.node_attributes);
            node
        };
        let backend = match config.backend {
            BackendConfig::Zookeeper(zookeeper) => Arc::new(
                backend::zookeeper::Zookeeper::new(node_id, zookeeper, logger)?
            ),
        };
        Ok(Coordinator(backend))
    }

    /// Internal method to create a `Coordinator` from the given backend.
    pub(crate) fn with_backend(backend: Arc<Backend>) -> Coordinator {
        Coordinator(backend)
    }
}

impl Coordinator {
    /// Get the ID of the current node.
    pub fn node_id(&self) -> &NodeId {
        self.0.node_id()
    }

    /// Interact with the `Tombstone` with the given `key`.
    pub fn tombstone(&self, key: String) -> Tombstone {
        Tombstone::new(Arc::clone(&self.0), key)
    }
}

use std::sync::Arc;

use slog::Logger;

use super::BackendConfig;
use super::Config;
use super::NodeId;
use super::Result;

use super::backend;
use super::backend::BackendAdmin;

mod lock;

pub use self::lock::NonBlockingLock;
pub use self::lock::NonBlockingLocks;


/// Interface to admin distributed coordination services.
#[derive(Clone)]
pub struct Admin(Arc<dyn BackendAdmin>);

impl Admin {
    pub fn new(config: Config, logger: Logger) -> Result<Admin> {
        let backend = match config.backend {
            BackendConfig::Zookeeper(zookeeper) => Arc::new(
                backend::zookeeper::ZookeeperAdmin::new(zookeeper, logger)?
            ),
        };
        Ok(Admin(backend))
    }

    /// Internal method to create an `Admin` from the given backend.
    pub(crate) fn with_backend(backend: Arc<dyn BackendAdmin>) -> Admin {
        Admin(backend)
    }
}

impl Admin {
    /// Iterate over registered nodes.
    pub fn nodes(&self) -> Nodes {
        self.0.nodes()
    }

    /// Model a non-blocking lock.
    pub fn non_blocking_lock(&self, lock: &str) -> Result<NonBlockingLock> {
        self.0.non_blocking_lock(lock)
    }

    /// Iterate over held non-blocking locks.
    pub fn non_blocking_locks(&self) -> NonBlockingLocks {
        self.0.non_blocking_locks()
    }
}


/// Iterator over nodes registered in the coordinator.
pub struct Nodes(Box<dyn Iterator<Item=Result<NodeId>>>);

impl Nodes {
    pub(crate) fn new<I: Iterator<Item=Result<NodeId>> + 'static>(iter: I) -> Nodes {
        Nodes(Box::new(iter))
    }
}

impl Iterator for Nodes {
    type Item = Result<NodeId>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

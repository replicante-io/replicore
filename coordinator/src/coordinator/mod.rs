use std::sync::Arc;

use slog::Logger;

use super::BackendConfig;
use super::Config;
use super::NodeId;
use super::Result;
use super::backend;
use super::backend::Backend;

mod lock;

pub use self::lock::NonBlockingLock;
pub use self::lock::NonBlockingLockWatcher;


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

    /// Return a non-blocking lock that can be acquaired/released as needed.
    ///
    /// If a lock is alreadt held by a process (including the current process)
    /// any acquire operation will fail.
    /// Only locks that are currently held can be released.
    ///
    /// Locks are automatically released if the process that holds them crashes 
    /// (or is no longer able to talk to the coordination system).
    /// 
    /// If a lock is lost (the coordinator is no longer reachable or thinks we no longer
    /// hold the lock for any reason) the state is changed and applications can check this.
    pub fn non_blocking_lock<S: Into<String>>(&self, lock: S) -> NonBlockingLock {
        self.0.non_blocking_lock(lock.into())
    }
}

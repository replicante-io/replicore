use std::sync::Arc;

use opentracingrust::Tracer;
use slog::Logger;

use replicante_service_healthcheck::HealthChecks;

use super::backend;
use super::backend::Backend;
use super::BackendConfig;
use super::Config;
use super::NodeId;
use super::Result;

mod election;
mod lock;
mod looping_election;

pub use self::election::Election;
pub use self::election::ElectionStatus;
pub use self::election::ElectionWatch;
pub use self::lock::NonBlockingLock;
pub use self::lock::NonBlockingLockWatcher;
pub use self::looping_election::LoopingElection;
pub use self::looping_election::LoopingElectionControl;
pub use self::looping_election::LoopingElectionLogic;
pub use self::looping_election::LoopingElectionOpts;
pub use self::looping_election::ShutdownReceiver;
pub use self::looping_election::ShutdownSender;

/// Interface to access distributed coordination services.
#[derive(Clone)]
pub struct Coordinator(Arc<dyn Backend>);

impl Coordinator {
    pub fn new<T>(
        config: Config,
        logger: Logger,
        healthchecks: &mut HealthChecks,
        tracer: T,
    ) -> Result<Coordinator>
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let node_id = {
            let mut node = NodeId::new();
            node.extra(config.node_attributes);
            node
        };
        let backend = match config.backend {
            BackendConfig::Zookeeper(zookeeper) => Arc::new(backend::zookeeper::Zookeeper::new(
                node_id,
                zookeeper,
                logger,
                healthchecks,
                tracer,
            )?),
        };
        Ok(Coordinator(backend))
    }

    /// Internal method to create a `Coordinator` from the given backend.
    #[cfg(debug_assertions)]
    pub(crate) fn with_backend(backend: Arc<dyn Backend>) -> Coordinator {
        Coordinator(backend)
    }
}

impl Coordinator {
    /// Election for a single primary with secondaries ready to take over.
    pub fn election<S>(&self, id: S) -> Election
    where
        S: Into<String>,
    {
        self.0.election(id.into())
    }

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
    pub fn non_blocking_lock<S>(&self, lock: S) -> NonBlockingLock
    where
        S: Into<String>,
    {
        self.0.non_blocking_lock(lock.into())
    }
}

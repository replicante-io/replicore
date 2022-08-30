use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;

use super::backend::ElectionBehaviour;
use super::coordinator::ElectionStatus;
use super::coordinator::ElectionWatch;
use super::Admin;
use super::Coordinator;
use super::ErrorKind;
use super::NodeId;
use super::Result;

mod admin;
mod backend;

use self::admin::MockAdmin;
use self::backend::MockBackend;

/// Helper to mock distributed coordination services.
#[derive(Default)]
pub struct MockCoordinator {
    pub elections: Arc<Mutex<HashMap<String, MockElection>>>,
    pub nblocks: Arc<Mutex<HashMap<String, MockNonBlockingLock>>>,
    pub node_id: NodeId,
}

impl MockCoordinator {
    pub fn new() -> MockCoordinator {
        MockCoordinator {
            elections: Arc::new(Mutex::new(HashMap::new())),
            nblocks: Arc::new(Mutex::new(HashMap::new())),
            node_id: NodeId::new(),
        }
    }

    pub fn admin(&self) -> Admin {
        Admin::with_backend(Arc::new(MockAdmin {
            nblocks: Arc::clone(&self.nblocks),
        }))
    }

    pub fn election<S: Into<String>>(&self, name: S) -> MockElection {
        let name: String = name.into();
        let mut elections = self
            .elections
            .lock()
            .expect("MockCoordinator::elections lock poisoned");
        elections
            .entry(name.clone())
            .or_insert_with(|| MockElection::new(name))
            .clone()
    }

    pub fn mock(&self) -> Coordinator {
        Coordinator::with_backend(Arc::new(MockBackend {
            elections: Arc::clone(&self.elections),
            nblocks: Arc::clone(&self.nblocks),
            node_id: self.node_id.clone(),
        }))
    }

    /// Get a mocked non-blocking lock for assertions and manipulation.
    pub fn non_blocking_lock<S: Into<String>>(&self, lock: S) -> MockNonBlockingLock {
        let lock = lock.into();
        let mut guard = self
            .nblocks
            .lock()
            .expect("MockCoordinator::nblocks poisoned");
        let mock = guard.get(&lock).map(Clone::clone);
        match mock {
            Some(mock) => mock,
            None => {
                let mock = MockNonBlockingLock::new(lock.clone(), self.node_id.clone());
                guard.insert(lock, mock.clone());
                mock
            }
        }
    }
}

/// Election mock behaviour.
#[derive(Clone)]
pub struct MockElection {
    #[allow(dead_code)]
    name: String,
    pub primary: Arc<Mutex<Option<NodeId>>>,
    pub secondaries: Arc<AtomicUsize>,
    pub status: Arc<Mutex<ElectionStatus>>,
}

impl MockElection {
    fn new(name: String) -> MockElection {
        MockElection {
            name,
            primary: Arc::new(Mutex::new(None)),
            secondaries: Arc::new(AtomicUsize::new(0)),
            status: Arc::new(Mutex::new(ElectionStatus::NotCandidate)),
        }
    }
}

impl ElectionBehaviour for MockElection {
    fn run(&mut self) -> Result<()> {
        let primary = self
            .primary
            .lock()
            .expect("MockElection::primary lock poisoned");
        let primary = primary.is_some();
        let status = if primary {
            ElectionStatus::Primary
        } else {
            ElectionStatus::Secondary
        };
        let mut lock = self
            .status
            .lock()
            .expect("MockElection::status lock poisoned");
        *lock = status;
        Ok(())
    }

    fn status(&self) -> ElectionStatus {
        let lock = self
            .status
            .lock()
            .expect("MockElection::status lock poisoned");
        lock.clone()
    }

    fn step_down(&mut self) -> Result<()> {
        let mut lock = self
            .status
            .lock()
            .expect("MockElection::status lock poisoned");
        *lock = ElectionStatus::NotCandidate;
        Ok(())
    }

    fn step_down_on_drop(&mut self) {
        self.step_down().expect("MockElection::step_down failed")
    }

    fn watch(&self) -> ElectionWatch {
        panic!("TODO: MockElection::watch");
    }
}

/// A mocked non-blocking lock for assertions and manipulation.
#[derive(Clone)]
pub struct MockNonBlockingLock {
    lock_id: String,
    locked: Arc<AtomicBool>,
    node_id: NodeId,
}

impl MockNonBlockingLock {
    pub fn new(lock_id: String, node_id: NodeId) -> MockNonBlockingLock {
        MockNonBlockingLock {
            lock_id,
            locked: Arc::new(AtomicBool::new(false)),
            node_id,
        }
    }
}

impl MockNonBlockingLock {
    pub fn acquire(&self) -> Result<()> {
        let before = self.locked.swap(true, Ordering::Relaxed);
        if before {
            return Err(ErrorKind::LockHeld(self.lock_id.clone(), self.node_id.clone()).into());
        }
        Ok(())
    }

    pub fn locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id.clone()
    }

    pub fn release(&self) -> Result<()> {
        let before = self.locked.swap(false, Ordering::Relaxed);
        if !before {
            return Err(ErrorKind::LockNotHeld(self.lock_id.clone(), self.node_id.clone()).into());
        }
        Ok(())
    }
}

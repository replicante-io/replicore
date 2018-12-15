use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use slog::Logger;

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
pub struct MockCoordinator {
    pub nblocks: Arc<Mutex<HashMap<String, MockNonBlockingLock>>>,
    pub node_id: NodeId,
}

impl MockCoordinator {
    pub fn new(_logger: Logger) -> MockCoordinator {
        MockCoordinator {
            nblocks: Arc::new(Mutex::new(HashMap::new())),
            node_id: NodeId::new(),
        }
    }

    pub fn admin(&self) -> Admin {
        Admin::with_backend(Arc::new(MockAdmin {
            nblocks: Arc::clone(&self.nblocks),
        }))
    }

    pub fn mock(&self) -> Coordinator {
        Coordinator::with_backend(Arc::new(MockBackend {
            nblocks: self.nblocks.clone(),
            node_id: self.node_id.clone(),
        }))
    }

    /// Get a mocked non-blocking lock for assertions and manipulation.
    pub fn non_blocking_lock<S: Into<String>>(&self, lock: S) -> MockNonBlockingLock {
        let lock = lock.into();
        let mut guard = self.nblocks.lock().expect("MockCoordinator::nblocks poisoned");
        let mock = guard.get(&lock).map(Clone::clone);
        match mock {
            None => {
                let mock = MockNonBlockingLock::new(lock.clone(), self.node_id.clone());
                guard.insert(lock, mock.clone());
                mock
            },
            Some(mock) => mock,
        }
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

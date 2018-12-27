use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use super::super::NodeId;
use super::super::Result;
use super::super::backend::Backend;
use super::super::backend::NonBlockingLockBehaviour;
use super::super::coordinator::Election;
use super::super::coordinator::NonBlockingLock;
use super::super::coordinator::NonBlockingLockWatcher;
use super::MockNonBlockingLock;



/// Proxy synchronized access to mock attributes.
pub struct MockBackend {
    pub nblocks: Arc<Mutex<HashMap<String, MockNonBlockingLock>>>,
    pub node_id: NodeId,
}

impl Backend for MockBackend {
    fn election(&self, _id: String) -> Election {
        panic!("TODO: MockBackend::election");
    }

    fn non_blocking_lock(&self, lock: String) -> NonBlockingLock {
        NonBlockingLock::new(Box::new(MockNBL {
            lock,
            nblocks: Arc::clone(&self.nblocks),
            node_id: self.node_id.clone(),
        }))
    }

    fn node_id(&self) -> &NodeId {
        &self.node_id
    }
}


/// Non-blocking lock mock behaviour.
struct MockNBL {
    lock: String,
    nblocks: Arc<Mutex<HashMap<String, MockNonBlockingLock>>>,
    node_id: NodeId,
}

impl NonBlockingLockBehaviour for MockNBL {
    fn acquire(&mut self) -> Result<()> {
        let mut guard = self.nblocks.lock().expect("MockBackend::nblocks poisoned");
        let mock = guard.get(&self.lock).map(Clone::clone);
        match mock {
            None => {
                let mock = MockNonBlockingLock::new(self.lock.clone(), self.node_id.clone());
                mock.acquire()?;
                guard.insert(self.lock.clone(), mock);
                Ok(())
            },
            Some(mock) => mock.acquire(),
        }
    }

    fn check(&self) -> bool {
        let guard = self.nblocks.lock().expect("MockBackend::nblocks poisoned");
        let mock = guard.get(&self.lock).map(Clone::clone);
        match mock {
            None => false,
            Some(mock) => mock.locked(),
        }
    }

    fn release(&mut self) -> Result<()> {
        let mut guard = self.nblocks.lock().expect("MockBackend::nblocks poisoned");
        let mock = guard.get(&self.lock).map(Clone::clone);
        match mock {
            None => {
                let mock = MockNonBlockingLock::new(self.lock.clone(), self.node_id.clone());
                guard.insert(self.lock.clone(), mock);
                Ok(())
            },
            Some(mock) => mock.release(),
        }
    }

    fn release_on_drop(&mut self) -> () {
        let guard = self.nblocks.lock().expect("MockBackend::nblocks poisoned");
        let mock = guard.get(&self.lock).map(Clone::clone);
        match mock {
            None => (),
            Some(mock) => {
                let _ = mock.release();
            },
        }
    }

    fn watch(&self) -> NonBlockingLockWatcher {
        let mut guard = self.nblocks.lock().expect("MockBackend::nblocks poisoned");
        let mock = guard.get(&self.lock).map(Clone::clone);
        let arc = match mock {
            None => {
                let mock = MockNonBlockingLock::new(self.lock.clone(), self.node_id.clone());
                let arc = Arc::clone(&mock.locked);
                guard.insert(self.lock.clone(), mock);
                arc
            },
            Some(mock) => Arc::clone(&mock.locked),
        };
        NonBlockingLockWatcher::new(arc)
    }
}

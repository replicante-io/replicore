use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use super::super::NodeId;
use super::super::Result;
use super::super::backend::Backend;
use super::super::backend::NonBlockingLockBehaviour;
use super::super::coordinator::NonBlockingLock;
use super::MockNonBlockingLock;



/// Proxy synchronized access to mock attributes.
pub struct MockBackend {
    pub nblocks: Arc<Mutex<HashMap<String, MockNonBlockingLock>>>,
    pub node_id: NodeId,
}

impl Backend for MockBackend {
    fn non_blocking_lock(&self, lock: String) -> NonBlockingLock {
        NonBlockingLock::new(Arc::new(MockNBL {
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
    fn acquire(&self) -> Result<()> {
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

    fn release(&self) -> Result<()> {
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

    fn release_on_drop(&self) -> () {
        let guard = self.nblocks.lock().expect("MockBackend::nblocks poisoned");
        let mock = guard.get(&self.lock).map(Clone::clone);
        match mock {
            None => (),
            Some(mock) => {
                let _ = mock.release();
            },
        }
    }
}

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use super::super::NodeId;
use super::super::Result;
use super::super::admin::Nodes;
use super::super::admin::NonBlockingLock;
use super::super::admin::NonBlockingLocks;
use super::super::backend::BackendAdmin;
use super::super::backend::NonBlockingLockAdminBehaviour;

use super::MockNonBlockingLock;


/// Proxy synchronized access to mock attributes.
pub struct MockAdmin {
    pub nblocks: Arc<Mutex<HashMap<String, MockNonBlockingLock>>>,
}

impl BackendAdmin for MockAdmin {
    fn nodes(&self) -> Nodes {
        Nodes::new(MockNodes {})
    }

    fn non_blocking_locks(&self) -> NonBlockingLocks {
        let nblocks: Vec<_> = self.nblocks.lock().expect("MockAdmin::nblocks poisoned")
            .iter()
            .map(|(k, v)| NonBlockingLock::new(k.to_string(), Arc::new(MockNBLAdmin {
                lock: v.clone(),
            })))
            .collect();
        let nblocks = nblocks.into_iter();
        NonBlockingLocks::new(MockNBLs {
            nblocks,
        })
    }
}


/// Iterate over nodes in the mock backend.
struct MockNodes {}

impl Iterator for MockNodes {
    type Item = Result<NodeId>;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}


/// Mock non-blocking lock admin behaviours.
struct MockNBLAdmin {
    lock: MockNonBlockingLock,
}

impl NonBlockingLockAdminBehaviour for MockNBLAdmin {
    fn force_release(&self) -> Result<()> {
        self.lock.release()
    }

    fn owner(&self) -> Result<NodeId> {
        Ok(self.lock.node_id())
    }
}


/// Iterate over held non-blocking locks in the mock backend.
struct MockNBLs {
    nblocks: ::std::vec::IntoIter<NonBlockingLock>,
}

impl Iterator for MockNBLs {
    type Item = Result<NonBlockingLock>;
    fn next(&mut self) -> Option<Self::Item> {
        self.nblocks.next().map(Ok)
    }
}

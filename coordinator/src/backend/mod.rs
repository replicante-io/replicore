use super::NodeId;
use super::Result;
use super::lock::NonBlockingLock;


pub mod zookeeper;


/// Distributed coordination backend interface.
pub trait Backend : Send + Sync {
    /// Get the ID of the current node.
    fn node_id(&self) -> &NodeId;

    /// Return a non-blocking lock that can be acquired/released as needed.
    fn non_blocking_lock(&self, lock: String) -> NonBlockingLock;
}


/// Distributed coordination admin backend interface.
pub trait BackendAdmin : Send + Sync {
    /// Iterate over registered nodes.
    fn nodes(&self) -> Nodes;
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


/// Backend specific non-blocking lock behaviours.
pub trait NonBlockingLockBehaviour {
    /// Attempt to acquire a non-blocking lock.
    fn acquire(&self) -> Result<()>;

    /// Lightweight check if the lock is held by us.
    fn check(&self) -> bool;

    /// Attempt to release a non-blocking lock.
    fn release(&self) -> Result<()>;

    /// Attempt to release a non-blocking lock when it is dropped.
    fn release_on_drop(&self) -> ();
}

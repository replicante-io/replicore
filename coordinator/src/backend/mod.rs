use super::NodeId;
use super::Result;
use super::admin::Nodes;
use super::admin::NonBlockingLocks;
use super::coordinator::NonBlockingLock;


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

    /// Iterate over held non-blocking locks.
    fn non_blocking_locks(&self) -> NonBlockingLocks;
}


/// Backend specific non-blocking lock admin behaviours.
pub trait NonBlockingLockAdminBehaviour {
    /// Attempt to release a non-blocking lock held by someone else.
    fn force_release(&self) -> Result<()>;

    /// Return the NodeId that owns a lock, if the lock is held.
    fn owner(&self) -> Result<NodeId>;
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

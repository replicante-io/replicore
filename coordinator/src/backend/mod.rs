use super::NodeId;
use super::Result;
use super::admin::Election as AdminElection;
use super::admin::Elections as AdminElections;
use super::admin::Nodes;
use super::admin::NonBlockingLock as AdminNonBlockingLock;
use super::admin::NonBlockingLocks;
use super::coordinator::Election;
use super::coordinator::ElectionStatus;
use super::coordinator::ElectionWatch;
use super::coordinator::NonBlockingLock;
use super::coordinator::NonBlockingLockWatcher;


pub mod zookeeper;


/// Distributed coordination backend interface.
pub trait Backend : Send + Sync {
    /// Election for a single primary with secondaries ready to take over.
    fn election(&self, id: String) -> Election;

    /// Get the ID of the current node.
    fn node_id(&self) -> &NodeId;

    /// Return a non-blocking lock that can be acquired/released as needed.
    fn non_blocking_lock(&self, lock: String) -> NonBlockingLock;
}


/// Distributed coordination admin backend interface.
pub trait BackendAdmin : Send + Sync {
    /// Lookup an election.
    fn election(&self, &str) -> Result<AdminElection>;

    /// Iterate over elections.
    fn elections(&self) -> AdminElections;

    /// Iterate over registered nodes.
    fn nodes(&self) -> Nodes;

    /// Lookup a non-blocking lock.
    fn non_blocking_lock(&self, lock: &str) -> Result<AdminNonBlockingLock>;

    /// Iterate over held non-blocking locks.
    fn non_blocking_locks(&self) -> NonBlockingLocks;
}


/// Backend specific elections admin behaviour.
pub trait ElectionAdminBehaviour {
    /// Fetch the `NodeId` of the primary for this election, if a primary is elected.
    fn primary(&self) -> Result<Option<NodeId>>;

    /// The number of secondary nodes waiting to take over if needed.
    fn secondaries_count(&self) -> Result<usize>;

    /// Strip the current primary of its role and forces a new election.
    fn step_down(&self) -> Result<bool>;
}


/// Backend specific elections behaviours.
pub trait ElectionBehaviour {
    /// Run for election.
    fn run(&mut self) -> Result<()>;

    /// Check the current election status.
    fn status(&self) -> ElectionStatus;

    /// Relinquish primary role, if primary, and remove itself from the election.
    fn step_down(&mut self) -> Result<()>;

    /// Step down logic called when the election instance is `Drop::drop`ed.
    fn step_down_on_drop(&mut self);

    /// Watch the election for changes.
    fn watch(&self) -> ElectionWatch;
}


/// Backend specific non-blocking lock admin behaviours.
pub trait NonBlockingLockAdminBehaviour {
    /// Attempt to release a non-blocking lock held by someone else.
    fn force_release(&mut self) -> Result<()>;

    /// Return the NodeId that owns a lock, if the lock is held.
    fn owner(&self) -> Result<NodeId>;
}


/// Backend specific non-blocking lock behaviours.
pub trait NonBlockingLockBehaviour {
    /// Attempt to acquire a non-blocking lock.
    fn acquire(&mut self) -> Result<()>;

    /// Lightweight check if the lock is held by us.
    fn check(&self) -> bool {
        self.watch().inspect()
    }

    /// Attempt to release a non-blocking lock.
    fn release(&mut self) -> Result<()>;

    /// Attempt to release a non-blocking lock when it is dropped.
    fn release_on_drop(&mut self) -> ();

    /// Return a shared `AtomicBool` that is kept in sync with the state of the lock.
    ///
    /// This can be used by applications to pass around a "flag" that matches the state of the
    /// lock and can be used to change course of action in case the lock is lost.
    ///
    ///   * A `true` value indicates the lock is held.
    ///   * A `false` value indicates the lock is NOT held.
    fn watch(&self) -> NonBlockingLockWatcher;
}

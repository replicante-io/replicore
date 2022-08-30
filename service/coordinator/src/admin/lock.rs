use super::super::backend::NonBlockingLockAdminBehaviour;
use super::super::NodeId;
use super::super::Result;

/// Admin tools for a non-blocking lock.
pub struct NonBlockingLock {
    behaviour: Box<dyn NonBlockingLockAdminBehaviour>,
    name: String,
}

impl NonBlockingLock {
    pub(crate) fn new(
        name: String,
        behaviour: Box<dyn NonBlockingLockAdminBehaviour>,
    ) -> NonBlockingLock {
        NonBlockingLock { behaviour, name }
    }
}

impl NonBlockingLock {
    pub fn force_release(&mut self) -> Result<()> {
        self.behaviour.force_release()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn owner(&self) -> Result<NodeId> {
        self.behaviour.owner()
    }
}

/// Iterator over held non-blocking locks.
pub struct NonBlockingLocks(Box<dyn Iterator<Item = Result<NonBlockingLock>>>);

impl NonBlockingLocks {
    pub(crate) fn new<I: Iterator<Item = Result<NonBlockingLock>> + 'static>(iter: I) -> Self {
        NonBlockingLocks(Box::new(iter))
    }
}

impl Iterator for NonBlockingLocks {
    type Item = Result<NonBlockingLock>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::mock::MockCoordinator;

    #[test]
    fn force_remove() {
        let mock_coordinator = MockCoordinator::default();
        let coordinator = mock_coordinator.mock();
        let mut held_lock = coordinator.non_blocking_lock("some/test/lock");
        held_lock.acquire(None).unwrap();
        let admin = mock_coordinator.admin();
        let mut locks = admin.non_blocking_locks();
        let mut lock = locks.next().unwrap().unwrap();
        assert_eq!("some/test/lock", lock.name());
        lock.force_release().unwrap();
        assert_eq!(false, held_lock.check());
        assert_eq!(true, locks.next().is_none());
    }

    #[test]
    fn owner() {
        let mock_coordinator = MockCoordinator::default();
        let coordinator = mock_coordinator.mock();
        let node_id = coordinator.node_id().clone();
        let mut held_lock = coordinator.non_blocking_lock("some/test/lock");
        held_lock.acquire(None).unwrap();
        let admin = mock_coordinator.admin();
        let mut locks = admin.non_blocking_locks();
        let lock = locks.next().unwrap().unwrap();
        assert_eq!("some/test/lock", lock.name());
        assert_eq!(node_id, lock.owner().unwrap());
        assert_eq!(true, locks.next().is_none());
    }
}

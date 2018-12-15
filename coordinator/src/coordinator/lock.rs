use std::sync::Arc;

use super::super::Result;
use super::super::backend::NonBlockingLockBehaviour;


/// A non-blocking lock that can be acquired/released as needed.
///
/// If a lock is already held by a process (including the current process)
/// any acquire operation will fail.
/// Only locks that are currently held can be released.
///
/// Locks are automatically released if the process that holds them crashes 
/// (or is no longer able to talk to the coordination system).
/// 
/// If a lock is lost (the coordinator is no longer reachable or thinks we no longer
/// hold the lock for any reason) the state is changed and applications can check this.
pub struct NonBlockingLock {
    behaviour: Arc<dyn NonBlockingLockBehaviour>,
}

impl NonBlockingLock {
    pub(crate) fn new(behaviour: Arc<dyn NonBlockingLockBehaviour>) -> NonBlockingLock {
        NonBlockingLock {
            behaviour,
        }
    }
}

impl NonBlockingLock {
    /// Attempt to acquire the named lock.
    pub fn acquire(&self) -> Result<()> {
        self.behaviour.acquire()
    }

    /// Lightweight check if the lock is held by us.
    pub fn check(&self) -> bool {
        self.behaviour.check()
    }

    /// Attempt to release the named lock.
    pub fn release(&self) -> Result<()> {
        self.behaviour.release()
    }
}

impl Drop for NonBlockingLock {
    fn drop(&mut self) {
        self.behaviour.release_on_drop();
    }
}


#[cfg(test)]
mod tests {
    use super::super::super::mock::MockCoordinator;
    use super::super::super::ErrorKind;

    fn mock_coordinator() -> MockCoordinator {
        let logger = ::slog::Logger::root(::slog::Discard, o!());
        MockCoordinator::new(logger)
    }

    #[test]
    fn acquire() {
        let mock_coordinator = mock_coordinator();
        let coordinator = mock_coordinator.mock();
        let lock = coordinator.non_blocking_lock("some/test/lock");
        let mock = mock_coordinator.non_blocking_lock("some/test/lock");
        assert_eq!(mock.locked(), false);
        lock.acquire().expect("lock to be acquired successfully");
        assert_eq!(mock.locked(), true);
    }

    #[test]
    fn acquire_locked_fails() {
        let mock_coordinator = mock_coordinator();
        let coordinator = mock_coordinator.mock();
        let lock1 = coordinator.non_blocking_lock("some/test/lock");
        let lock2 = coordinator.non_blocking_lock("some/test/lock");
        let mock = mock_coordinator.non_blocking_lock("some/test/lock");
        assert_eq!(mock.locked(), false);
        lock1.acquire().expect("lock to be acquired successfully");
        assert_eq!(mock.locked(), true);
        match lock2.acquire() {
            Ok(()) => panic!("lock acquired twice"),
            Err(error) => {
                match error.kind() {
                    ErrorKind::LockHeld(_, _) => (),
                    error => panic!("{}", error),
                }
            }
        }
    }

    #[test]
    fn check() {
        let mock_coordinator = mock_coordinator();
        let coordinator = mock_coordinator.mock();
        let lock = coordinator.non_blocking_lock("some/test/lock");
        let mock = mock_coordinator.non_blocking_lock("some/test/lock");
        assert_eq!(mock.locked(), false);
        assert_eq!(false, lock.check());
        lock.acquire().expect("lock to be acquired successfully");
        assert_eq!(mock.locked(), true);
        assert_eq!(true, lock.check());
        lock.release().expect("lock to be released successfully");
        assert_eq!(mock.locked(), false);
        assert_eq!(false, lock.check());
    }

    #[test]
    fn release() {
        let mock_coordinator = mock_coordinator();
        let coordinator = mock_coordinator.mock();
        let lock = coordinator.non_blocking_lock("some/test/lock");
        let mock = mock_coordinator.non_blocking_lock("some/test/lock");
        assert_eq!(mock.locked(), false);
        lock.acquire().expect("lock to be acquired successfully");
        assert_eq!(mock.locked(), true);
        lock.release().expect("lock to be released successfully");
        assert_eq!(mock.locked(), false);
    }

    #[test]
    fn release_on_drop() {
        let mock_coordinator = mock_coordinator();
        let coordinator = mock_coordinator.mock();
        let mock = mock_coordinator.non_blocking_lock("some/test/lock");
        {
            let lock = coordinator.non_blocking_lock("some/test/lock");
            assert_eq!(mock.locked(), false);
            lock.acquire().expect("lock to be acquired successfully");
            assert_eq!(mock.locked(), true);
        }
        assert_eq!(mock.locked(), false);
    }

    #[test]
    fn release_unlocked_fails() {
        let mock_coordinator = mock_coordinator();
        let coordinator = mock_coordinator.mock();
        let lock = coordinator.non_blocking_lock("some/test/lock");
        let mock = mock_coordinator.non_blocking_lock("some/test/lock");
        assert_eq!(mock.locked(), false);
        lock.acquire().expect("lock to be acquired successfully");
        assert_eq!(mock.locked(), true);
        lock.release().expect("lock to be released successfully");
        assert_eq!(mock.locked(), false);
        match lock.release() {
            Ok(()) => panic!("lock released twice"),
            Err(error) => {
                match error.kind() {
                    ErrorKind::LockNotHeld(_, _) => (),
                    ErrorKind::LockNotFound(_) => (),
                    error => panic!("{}", error),
                }
            }
        }
    }
}

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use opentracingrust::SpanContext;

use super::super::backend::NonBlockingLockBehaviour;
use super::super::Result;

use super::super::metrics::NB_LOCK_ACQUIRE_FAIL;
use super::super::metrics::NB_LOCK_ACQUIRE_TOTAL;
use super::super::metrics::NB_LOCK_RELEASE_FAIL;
use super::super::metrics::NB_LOCK_RELEASE_TOTAL;

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
///
/// # A note on CAP
/// Please note that locks prefer strong consistency over availability.
/// As such, if the coordinator system fails and remains unavailable the lock will be released.
pub struct NonBlockingLock {
    behaviour: Box<dyn NonBlockingLockBehaviour>,
}

impl NonBlockingLock {
    pub(crate) fn new(behaviour: Box<dyn NonBlockingLockBehaviour>) -> NonBlockingLock {
        NonBlockingLock { behaviour }
    }
}

impl NonBlockingLock {
    /// Attempt to acquire the named lock.
    pub fn acquire<S>(&mut self, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        NB_LOCK_ACQUIRE_TOTAL.inc();
        self.behaviour.acquire(span.into()).map_err(|error| {
            NB_LOCK_ACQUIRE_FAIL.inc();
            error
        })
    }

    /// Lightweight check if the lock is held by us.
    pub fn check(&self) -> bool {
        self.behaviour.check()
    }

    /// Attempt to release the named lock.
    pub fn release<S>(&mut self, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        NB_LOCK_RELEASE_TOTAL.inc();
        self.behaviour.release(span.into()).map_err(|error| {
            NB_LOCK_RELEASE_FAIL.inc();
            error
        })
    }

    /// Return a watcher that is kept in sync with the state of the lock.
    ///
    /// This can be used by applications to pass around a "flag" that matches the state of the
    /// lock and can be used to change course of action in case the lock is lost.
    pub fn watch(&self) -> NonBlockingLockWatcher {
        self.behaviour.watch()
    }
}

impl Drop for NonBlockingLock {
    fn drop(&mut self) {
        self.behaviour.release_on_drop();
    }
}

/// Watcher of a non-blocking lock returned by `NonBlockingLock::watch`.
pub struct NonBlockingLockWatcher(Arc<AtomicBool>);

impl NonBlockingLockWatcher {
    pub(crate) fn new(watcher: Arc<AtomicBool>) -> NonBlockingLockWatcher {
        NonBlockingLockWatcher(watcher)
    }

    /// Inspect the state of the lock.
    ///
    ///   * A `true` value indicates the lock is held.
    ///   * A `false` value indicates the lock is NOT held.
    pub fn inspect(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::mock::MockCoordinator;
    use super::super::super::ErrorKind;

    #[test]
    fn acquire() {
        let mock_coordinator = MockCoordinator::default();
        let coordinator = mock_coordinator.mock();
        let mut lock = coordinator.non_blocking_lock("some/test/lock");
        let mock = mock_coordinator.non_blocking_lock("some/test/lock");
        assert_eq!(mock.locked(), false);
        lock.acquire(None)
            .expect("lock to be acquired successfully");
        assert_eq!(mock.locked(), true);
    }

    #[test]
    fn acquire_locked_fails() {
        let mock_coordinator = MockCoordinator::default();
        let coordinator = mock_coordinator.mock();
        let mut lock1 = coordinator.non_blocking_lock("some/test/lock");
        let mut lock2 = coordinator.non_blocking_lock("some/test/lock");
        let mock = mock_coordinator.non_blocking_lock("some/test/lock");
        assert_eq!(mock.locked(), false);
        lock1
            .acquire(None)
            .expect("lock to be acquired successfully");
        assert_eq!(mock.locked(), true);
        match lock2.acquire(None) {
            Ok(()) => panic!("lock acquired twice"),
            Err(error) => match error.kind() {
                ErrorKind::LockHeld(_, _) => (),
                error => panic!("{}", error),
            },
        }
    }

    #[test]
    fn check() {
        let mock_coordinator = MockCoordinator::default();
        let coordinator = mock_coordinator.mock();
        let mut lock = coordinator.non_blocking_lock("some/test/lock");
        let mock = mock_coordinator.non_blocking_lock("some/test/lock");
        assert_eq!(mock.locked(), false);
        assert_eq!(false, lock.check());
        lock.acquire(None)
            .expect("lock to be acquired successfully");
        assert_eq!(mock.locked(), true);
        assert_eq!(true, lock.check());
        lock.release(None)
            .expect("lock to be released successfully");
        assert_eq!(mock.locked(), false);
        assert_eq!(false, lock.check());
    }

    #[test]
    fn release() {
        let mock_coordinator = MockCoordinator::default();
        let coordinator = mock_coordinator.mock();
        let mut lock = coordinator.non_blocking_lock("some/test/lock");
        let mock = mock_coordinator.non_blocking_lock("some/test/lock");
        assert_eq!(mock.locked(), false);
        lock.acquire(None)
            .expect("lock to be acquired successfully");
        assert_eq!(mock.locked(), true);
        lock.release(None)
            .expect("lock to be released successfully");
        assert_eq!(mock.locked(), false);
    }

    #[test]
    fn release_on_drop() {
        let mock_coordinator = MockCoordinator::default();
        let coordinator = mock_coordinator.mock();
        let mock = mock_coordinator.non_blocking_lock("some/test/lock");
        {
            let mut lock = coordinator.non_blocking_lock("some/test/lock");
            assert_eq!(mock.locked(), false);
            lock.acquire(None)
                .expect("lock to be acquired successfully");
            assert_eq!(mock.locked(), true);
        }
        assert_eq!(mock.locked(), false);
    }

    #[test]
    fn release_unlocked_fails() {
        let mock_coordinator = MockCoordinator::default();
        let coordinator = mock_coordinator.mock();
        let mut lock = coordinator.non_blocking_lock("some/test/lock");
        let mock = mock_coordinator.non_blocking_lock("some/test/lock");
        assert_eq!(mock.locked(), false);
        lock.acquire(None)
            .expect("lock to be acquired successfully");
        assert_eq!(mock.locked(), true);
        lock.release(None)
            .expect("lock to be released successfully");
        assert_eq!(mock.locked(), false);
        match lock.release(None) {
            Ok(()) => panic!("lock released twice"),
            Err(error) => match error.kind() {
                ErrorKind::LockNotHeld(_, _) => (),
                ErrorKind::LockNotFound(_) => (),
                error => panic!("{}", error),
            },
        }
    }
}

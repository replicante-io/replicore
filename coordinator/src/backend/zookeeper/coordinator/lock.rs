use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use failure::ResultExt;
use slog::Logger;

use zookeeper::Acl;
use zookeeper::CreateMode;
use zookeeper::Subscription;
use zookeeper::WatchedEvent;
use zookeeper::WatchedEventType;
use zookeeper::ZkError;
use zookeeper::ZkState;
use zookeeper::ZooKeeper;

use replicante_util_failure::failure_info;

use super::super::super::super::ErrorKind;
use super::super::super::super::NodeId;
use super::super::super::super::Result;
use super::super::super::super::coordinator::NonBlockingLockWatcher;

use super::super::super::super::metrics::NB_LOCK_LOST;
use super::super::super::super::metrics::NB_LOCK_DROP_FAIL;
use super::super::super::super::metrics::NB_LOCK_DROP_TOTAL;

use super::super::super::NonBlockingLockBehaviour;
use super::super::NBLockInfo;
use super::super::client::Client;
use super::super::constants::PREFIX_LOCK;

use super::super::metrics::ZOO_NB_LOCK_DELETED;
use super::super::metrics::ZOO_NB_LOCK_LOST;
use super::super::metrics::ZOO_OP_DURATION;
use super::super::metrics::ZOO_OP_ERRORS_COUNT;
use super::super::metrics::ZOO_TIMEOUTS_COUNT;


/// Zookeeper non-blocking lock behaviour code.
pub struct ZookeeperNBLock {
    context: NblCallbackContext,
    listener_id: Option<Subscription>,
    payload: NBLockInfo,
}

impl ZookeeperNBLock {
    pub fn new(
        client: Arc<Client>, lock: String, owner: NodeId, logger: Logger
    ) -> ZookeeperNBLock {
        let path = Client::path_from_key(PREFIX_LOCK, &lock);
        let payload = NBLockInfo {
            name: lock.clone(),
            owner,
        };
        let state = NblSyncState::new(lock);
        let context = NblCallbackContext {
            client,
            logger,
            path,
            state,
        };
        ZookeeperNBLock {
            context,
            listener_id: None,
            payload,
        }
    }
}

impl ZookeeperNBLock {
    /// Handle a zookeeper watch event.
    ///
    /// If the lock znode was delete, release the lock.
    /// Otherwise reset the watcher on the node to be notified of new events.
    fn callback_event(context: &NblCallbackContext, event: WatchedEvent) {
        // Do nothing if the lock is not acquired on znode delete (we released the lock).
        let (acquired, _, _) = context.state.inspect();
        if !acquired {
            return;
        }

        // Release the lock if the node was deleted.
        if let WatchedEventType::NodeDeleted = event.event_type {
            error!(
                context.logger, "Lock lost, znode was deleted";
                "lock" => &context.state.lock
            );
            context.state.release();
            ZOO_NB_LOCK_DELETED.inc();
            NB_LOCK_LOST.inc();
            return;
        }
        
        // Re-register lock watcher.
        let block = || -> Result<()> {
            let keeper = context.client.get()?;
            let inner_context = context.clone();
            let stats = keeper.exists_w(&context.path, move |event| {
                ZookeeperNBLock::callback_event(&inner_context, event);
            })
            .context(ErrorKind::Backend("lock watching"))?;

            // If the node was deleted before watching, release the lock.
            if let Some(_) = stats {
                debug!(
                    context.logger, "Refreshed non-blocking lock watcher";
                    "lock" => &context.state.lock
                );
                return Ok(());
            }
            error!(
                context.logger, "Lock lost, znode was deleted";
                "lock" => &context.state.lock
            );
            context.state.release();
            ZOO_NB_LOCK_DELETED.inc();
            NB_LOCK_LOST.inc();
            Ok(())
        };
        if let Err(error) = block() {
            context.state.release();
            ZOO_NB_LOCK_LOST.inc();
            NB_LOCK_LOST.inc();
            error!(
                context.logger, "Lock lost, failed to reattach change watcher";
                failure_info(&error)
            );
        }
    }

    /// Handle a client state notification.
    ///
    /// If the session was closed, release the lock.
    /// Otherwise do nothing.
    fn callback_state(context: &NblCallbackContext, status: ZkState) {
        if let ZkState::Closed = status {
            error!(
                context.logger, "Lock lost, zookeeper session expired";
                "lock" => &context.state.lock
            );
            context.state.release();
            ZOO_NB_LOCK_LOST.inc();
            NB_LOCK_LOST.inc();
        }
    }
}

impl ZookeeperNBLock {
    /// Create the lock znode or fail if the lock is taken.
    fn create(&self, keeper: &ZooKeeper, path: &str) -> Result<()> {
        let data = serde_json::to_vec(&self.payload)
            .context(ErrorKind::Encode("zookeeper non-blocking lock"))?;
        let timer = ZOO_OP_DURATION.with_label_values(&["create"]).start_timer();
        let result = keeper.create(path, data, Acl::read_unsafe().clone(), CreateMode::Ephemeral)
            .map_err(|error| {
                ZOO_OP_ERRORS_COUNT.with_label_values(&["create"]).inc();
                if error == ZkError::OperationTimeout {
                    ZOO_TIMEOUTS_COUNT.inc();
                }
                error
            });
        timer.observe_duration();
        match result {
            Ok(_) => (),
            Err(ZkError::NodeExists) => {
                let payload = self.read(keeper, path)?;
                let payload: NBLockInfo = serde_json::from_slice(&payload)
                    .context(ErrorKind::Decode("zookeeper non-blocking lock"))?;
                return Err(ErrorKind::LockHeld(
                    self.context.state.lock.clone(), payload.owner
                ).into());
            },
            Err(error) => {
                return Err(error).context(ErrorKind::Backend("lock acquisition"))?;
            }
        }
        Ok(())
    }

    /// Read the content of a znode.
    fn read(&self, keeper: &ZooKeeper, path: &str) -> Result<Vec<u8>> {
        let (data, _) = Client::get_data(keeper, path, false)
            .context(ErrorKind::Backend("lock read"))?;
        Ok(data)
    }

    /// Unsubscribe the zookeeper client listener, if any was set.
    fn unsubscribe(&mut self) {
        if let Some(listener_id) = self.listener_id.take() {
            match self.context.client.get() {
                Err(_) => (),
                Ok(keeper) => keeper.remove_listener(listener_id),
            };
        }
    }
}

impl NonBlockingLockBehaviour for ZookeeperNBLock {
    /// Attempt to acquire a lock.
    ///
    /// # Panics
    /// If attempting to acquire the lock while it is acquired.
    fn acquire(&mut self) -> Result<()> {
        let (acquired, _, version) = self.context.state.inspect();
        if acquired {
            panic!("Attempted to acquire held lock '{}'", self.context.state.lock);
        }
        let keeper = self.context.client.get()?;

        // Add listener to client for disconnect events
        let context = self.context.clone();
        let listener_id = keeper.add_listener(move |status| {
            ZookeeperNBLock::callback_state(&context, status);
        });
        self.listener_id = Some(listener_id);

        // Create ephimeral node for the lock.
        let dir = Client::container_path(&self.context.path);
        Client::mkcontaner(&keeper, &dir)?;
        self.create(&keeper, &self.context.path)?;

        // Check node and install delete + disconnect watcher.
        let context = self.context.clone();
        let stats = Client::exists_w(&keeper, &self.context.path, move |event| {
            ZookeeperNBLock::callback_event(&context, event);
        })
        .context(ErrorKind::Backend("lock watching"))?;
        let stats = match stats {
            Some(stats) => stats,
            None => {
                return Err(ErrorKind::LockLost(self.context.state.lock.clone()).into());
            }
        };

        self.context.state.acquire(stats.czxid, version)?;
        Ok(())
    }

    fn release(&mut self) -> Result<()> {
        self.unsubscribe();
        let (acquired, czxid, _) = self.context.state.inspect();
        if !acquired {
            return Ok(());
        }

        // Grab the session ID we used to create the lock.
        let czxid = czxid.expect("have an acquired lock without czxid");

        // Ensure we actually own the lock even though we shold release it if ever lost.
        let keeper = self.context.client.get()?;
        let stats = Client::exists(&keeper, &self.context.path, false)
            .context(ErrorKind::Backend("lock stats fetching"))?;
        self.context.state.release();
        match stats {
            None => (),
            Some(ref stats) if stats.czxid == czxid => {
                // Delete the lock znode and release the internal state.
                match keeper.delete(&self.context.path, None) {
                    Ok(()) => (),
                    Err(ZkError::NoNode) => (),
                    Err(error) => {
                        return Err(error).context(ErrorKind::Backend("lock release"))?;
                    }
                };
            },
            Some(_) => {
                // Lock exists, we think we own it but is not the one we created.
                let payload = self.read(&keeper, &self.context.path)?;
                let payload: NBLockInfo = serde_json::from_slice(&payload)
                    .context(ErrorKind::Decode("zookeeper non-blocking lock"))?;
                warn!(
                    self.context.logger, "Attempted lock release but we seem not to be owners";
                    "lock" => &self.context.state.lock, "owner" => %payload.owner
                );
            }
        };
        Ok(())
    }

    fn release_on_drop(&mut self) -> () {
        let (acquired, _, _) = self.context.state.inspect();
        if acquired {
            NB_LOCK_DROP_TOTAL.inc();
        }
        if let Err(error) = self.release() {
            NB_LOCK_DROP_FAIL.inc();
            error!(
                self.context.logger, "Unable to release lock from destructor";
                failure_info(&error)
            );
        }
    }

    fn watch(&self) -> NonBlockingLockWatcher {
        self.context.state.watch()
    }
}


/// Syncronised internal state for non-blocking locks.
///
/// The internal state of a ZookeeperNBLock object can be:
///
///   * `acquired` is false and `czxid` is None (the lock is no held).
///   * `acquired` is true and `czxid` is Some(zxid) (the lock is held by us).
#[derive(Clone)]
struct NblSyncState {
    inner: Arc<Mutex<NblSyncStateInner>>,
    lock: String
}

impl NblSyncState {
    fn new(lock: String) -> NblSyncState {
        let inner = Arc::new(Mutex::new(NblSyncStateInner {
            acquired: Arc::new(AtomicBool::new(false)),
            czxid: None,
            version: 0,
        }));
        NblSyncState {
            inner,
            lock,
        }
    }

    fn acquire(&self, czxid: i64, version: u64) -> Result<()> {
        let mut inner = self.inner.lock().expect("internal lock state poisoned");
        if inner.version != version {
            return Err(ErrorKind::LockLost(self.lock.clone()).into());
        }
        inner.acquired.store(true, Ordering::Relaxed);
        inner.czxid = Some(czxid);
        Ok(())
    }

    fn inspect(&self) -> (bool, Option<i64>, u64) {
        let inner = self.inner.lock().expect("internal lock state poisoned");
        (inner.acquired.load(Ordering::Relaxed), inner.czxid.clone(), inner.version)
    }

    fn release(&self) {
        let mut inner = self.inner.lock().expect("internal lock state poisoned");
        inner.acquired.store(false, Ordering::Relaxed);
        inner.czxid = None;
        inner.version += 1;
    }

    fn watch(&self) -> NonBlockingLockWatcher {
        let inner = self.inner.lock().expect("internal lock state poisoned");
        NonBlockingLockWatcher::new(Arc::clone(&inner.acquired))
    }
}


/// Inner non-blocking lock raw state.
struct NblSyncStateInner {
    acquired: Arc<AtomicBool>,
    czxid: Option<i64>,
    version: u64,
}


/// Collection of non-blocking lock state shared across the node and all callbacks.
#[derive(Clone)]
struct NblCallbackContext {
    client: Arc<Client>,
    logger: Logger,
    path: String,
    state: NblSyncState,
}
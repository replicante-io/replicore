use std::sync::Arc;

use failure::ResultExt;
use zookeeper::ZkError;

use super::super::super::super::ErrorKind;
use super::super::super::super::NodeId;
use super::super::super::super::Result;
use super::super::super::super::admin::NonBlockingLock;
use super::super::super::NonBlockingLockAdminBehaviour;
use super::super::NBLockInfo;
use super::super::client::Client;
use super::super::constants::PREFIX_LOCK;


/// Iterate over registered non-blocking locks.
pub struct ZookeeperNBLocks {
    pub(super) client: Arc<Client>,
    pub(super) locks: Option<Vec<String>>,
}

impl ZookeeperNBLocks {
    /// Enumerate all locks currently held in the coordinator.
    fn load_locks(&mut self) -> Result<()> {
        let keeper = self.client.get()?;
        let mut prefixes = Client::get_children(&keeper, PREFIX_LOCK, false)
            .context(ErrorKind::Backend("iterating over locks"))?;
        let mut locks = Vec::new();
        while let Some(prefix) = prefixes.pop() {
            let path = format!("{}/{}", PREFIX_LOCK, prefix);
            let mut nodes = Client::get_children(&keeper, &path, false)
                .context(ErrorKind::Backend("iterating over locks"))?;
            while let Some(node) = nodes.pop() {
                let lock = format!("{}/{}/{}", PREFIX_LOCK, prefix, node);
                locks.push(lock);
            }
        }
        self.locks = Some(locks);
        Ok(())
    }
}

impl Iterator for ZookeeperNBLocks {
    type Item = Result<NonBlockingLock>;
    fn next(&mut self) -> Option<Self::Item> {
        // Enumerate locks on the server.
        if self.locks.is_none() {
            if let Err(error) = self.load_locks() {
                // Cache an empty list to avoid endlessly attempting load after error.
                self.locks = Some(Vec::new());
                return Some(Err(error));
            }
        }

        // Process locks until all have been returned.
        // Locks for which a NoNode error is returned are ignored.
        let keeper = match self.client.get() {
            Ok(keeper) => keeper,
            Err(error) => return Some(Err(error)),
        };
        let locks = self.locks.as_mut().expect("ZookeeperNBLocks::locks must be Some(Vec)");
        while let Some(path) = locks.pop() {
            let lock = Client::get_data(&keeper, &path, false);
            let lock = match lock {
                Ok((lock, _)) => lock,
                Err(ZkError::NoNode) => continue,
                Err(error) => {
                    let error = Err(error).context(ErrorKind::Backend("iterating over locks"));
                    return Some(error.map_err(|e| e.into()));
                }
            };
            let lock: NBLockInfo = match serde_json::from_slice(&lock) {
                Ok(lock) => lock,
                Err(error) => {
                    let error = Err(error).context(ErrorKind::Decode("lock info"));
                    return Some(error.map_err(|e| e.into()));
                },
            };
            let name = lock.name.clone();
            let behaviour = ZookeeperNBLBehaviour {
                client: Arc::clone(&self.client),
                info: lock,
                path,
            };
            let lock = NonBlockingLock::new(name, Box::new(behaviour));
            return Some(Ok(lock));
        }
        None
    }
}


/// Admin behaviour for zookeeper non-blocking locks.
pub struct ZookeeperNBLBehaviour {
    pub(super) client: Arc<Client>,
    pub(super) info: NBLockInfo,
    pub(super) path: String,
}

impl NonBlockingLockAdminBehaviour for ZookeeperNBLBehaviour {
    fn force_release(&mut self) -> Result<()> {
        let keeper = self.client.get()?;
        match Client::delete(&keeper, &self.path, None) {
            Ok(()) => (),
            Err(ZkError::NoNode) => (),
            Err(error) => {
                let error = Err(error).context(ErrorKind::Backend("force-releasing lock"));
                return error.map_err(|e| e.into());
            }
        }
        Ok(())
    }

    fn owner(&self) -> Result<NodeId> {
        Ok(self.info.owner.clone())
    }
}

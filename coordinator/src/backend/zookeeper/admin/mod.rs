use std::sync::Arc;

use failure::ResultExt;
use slog::Logger;
use zookeeper::ZkError;
use zookeeper::ZooKeeper;

use super::super::super::admin::Election;
use super::super::super::admin::Elections;
use super::super::super::admin::NonBlockingLock;
use super::super::super::config::ZookeeperConfig;
use super::super::super::Error;
use super::super::super::ErrorKind;
use super::super::super::NodeId;
use super::super::super::Result;
use super::super::BackendAdmin;
use super::super::Nodes;
use super::super::NonBlockingLocks;
use super::client::Client;
use super::constants::PREFIX_LOCK;
use super::constants::PREFIX_NODE;
use super::NBLockInfo;

mod election;
mod lock;

/// Admin backend for zookeeper distributed coordination.
pub struct ZookeeperAdmin {
    client: Arc<Client>,
}

impl ZookeeperAdmin {
    pub fn new(config: ZookeeperConfig, logger: Logger) -> Result<ZookeeperAdmin> {
        let client = Arc::new(Client::new(config, None, logger)?);
        Ok(ZookeeperAdmin { client })
    }
}

impl BackendAdmin for ZookeeperAdmin {
    fn election(&self, name: &str) -> Result<Election> {
        match election::ZooKeeperElectionAdmin::from_name(Arc::clone(&self.client), name) {
            Err(error) => Err(error),
            Ok(Some(election)) => Ok(election),
            Ok(None) => Err(ErrorKind::ElectionNotFound(name.into()).into()),
        }
    }

    fn elections(&self) -> Elections {
        Elections::new(election::ZooKeeperElections::new(Arc::clone(&self.client)))
    }

    fn nodes(&self) -> Nodes {
        Nodes::new(ZookeeperNodes {
            client: Arc::clone(&self.client),
            nodes: None,
        })
    }

    fn non_blocking_lock(&self, lock: &str) -> Result<NonBlockingLock> {
        let keeper = self.client.get()?;
        let path = Client::path_from_key(PREFIX_LOCK, lock);
        let payload = Client::get_data(&keeper, &path, false);
        let payload = match payload {
            Ok((payload, _)) => payload,
            Err(ZkError::NoNode) => return Err(ErrorKind::LockNotFound(lock.to_string()).into()),
            Err(error) => {
                let error = Err(error).context(ErrorKind::Backend("non-blocking lock lookup"));
                return error.map_err(Error::from);
            }
        };
        let info: NBLockInfo = match serde_json::from_slice(&payload) {
            Ok(info) => info,
            Err(error) => {
                let error = Err(error).context(ErrorKind::Decode("lock info"));
                return error.map_err(Error::from);
            }
        };
        let name = info.name.clone();
        let behaviour = lock::ZookeeperNBLBehaviour {
            client: Arc::clone(&self.client),
            info,
            path,
        };
        let lock = NonBlockingLock::new(name, Box::new(behaviour));
        Ok(lock)
    }

    fn non_blocking_locks(&self) -> NonBlockingLocks {
        NonBlockingLocks::new(lock::ZookeeperNBLocks {
            client: Arc::clone(&self.client),
            locks: None,
        })
    }

    fn version(&self) -> Result<String> {
        Ok("Zookeeper (version not reported)".into())
    }
}

/// Iterate over nodes registered in zookeeper.
///
/// The list of nodes is fully loaded at the first iteration
/// but the details of each node are lazy loaded.
struct ZookeeperNodes {
    client: Arc<Client>,
    nodes: Option<Vec<String>>,
}

impl ZookeeperNodes {
    /// Load all known nodes in the cache.
    fn fill_cache(&mut self) -> Result<()> {
        let keeper = self.client.get()?;
        let mut all_nodes = Vec::new();
        let top_level = self.get_children(&keeper, PREFIX_NODE)?;

        // Load nested children.
        for top in top_level {
            let path = format!("{}/{}", PREFIX_NODE, top);
            let nodes = self.get_children(&keeper, &path)?;
            for node in nodes {
                let node = format!("{}/{}", path, node);
                all_nodes.push(node);
            }
        }

        all_nodes.reverse();
        self.nodes = Some(all_nodes);
        Ok(())
    }

    /// Wrapper to get children and track stats.
    fn get_children(&self, keeper: &ZooKeeper, path: &str) -> Result<Vec<String>> {
        let nodes = Client::get_children(keeper, path, false)
            .context(ErrorKind::Backend("nodes lookup"))?;
        Ok(nodes)
    }
}

impl Iterator for ZookeeperNodes {
    type Item = Result<NodeId>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.nodes.is_none() {
            if let Err(error) = self.fill_cache() {
                return Some(Err(error));
            }
        }
        let nodes = self.nodes.as_mut().unwrap();
        let keeper = match self.client.get() {
            Err(error) => return Some(Err(error)),
            Ok(keeper) => keeper,
        };

        // Ignore nodes that return ZkError::NoNode.
        while let Some(node) = nodes.pop() {
            let result = Client::get_data(&keeper, &node, false);
            let node = match result {
                Err(ZkError::NoNode) => continue,
                Err(error) => {
                    let error = Err(error).context(ErrorKind::Backend("node read"));
                    return Some(error.map_err(Error::from));
                }
                Ok((node, _)) => node,
            };
            let node: Result<NodeId> = match serde_json::from_slice(&node) {
                Err(error) => {
                    let error = Err(error).context(ErrorKind::Decode("node info"));
                    error.map_err(Error::from)
                }
                Ok(node) => Ok(node),
            };
            return Some(node);
        }
        None
    }
}

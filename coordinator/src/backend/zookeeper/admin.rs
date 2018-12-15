use std::sync::Arc;

use failure::ResultExt;
use slog::Logger;
use zookeeper::ZkError;
use zookeeper::ZooKeeper;

use super::super::super::Error;
use super::super::super::ErrorKind;
use super::super::super::NodeId;
use super::super::super::Result;
use super::super::super::config::ZookeeperConfig;
use super::super::BackendAdmin;
use super::super::Nodes;
use super::super::NonBlockingLocks;

use super::client::Client;
use super::metrics::ZOO_OP_DURATION;
use super::metrics::ZOO_OP_ERRORS_COUNT;
use super::metrics::ZOO_TIMEOUTS_COUNT;


/// Admin backend for zookeeper distributed coordination.
pub struct ZookeeperAdmin {
    client: Arc<Client>,
    //logger: Logger,
}

impl ZookeeperAdmin {
    pub fn new(config: ZookeeperConfig, logger: Logger) -> Result<ZookeeperAdmin> {
        let client = Arc::new(Client::new(config, None, logger.clone())?);
        Ok(ZookeeperAdmin {
            client,
            //logger,
        })
    }
}

impl BackendAdmin for ZookeeperAdmin {
    fn nodes(&self) -> Nodes {
        Nodes::new(ZookeeperNodes {
            client: Arc::clone(&self.client),
            nodes: None,
        })
    }

    fn non_blocking_locks(&self) -> NonBlockingLocks {
        panic!("TODO: ZookeeperAdmin::non_blocking_locks");
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
        let top_level = self.get_children(&keeper, "/nodes")?;

        // Load nested children.
        for top in top_level {
            let path = format!("/nodes/{}", top);
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
        let _timer = ZOO_OP_DURATION.with_label_values(&["get_children"]).start_timer();
        let nodes = keeper.get_children(path, false)
            .map_err(|error| {
                ZOO_OP_ERRORS_COUNT.with_label_values(&["get_children"]).inc();
                if error == ZkError::OperationTimeout {
                    ZOO_TIMEOUTS_COUNT.inc();
                }
                error
            })
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
            let timer = ZOO_OP_DURATION.with_label_values(&["get_data"]).start_timer();
            let result = keeper.get_data(&node, false).map_err(|error| {
                ZOO_OP_ERRORS_COUNT.with_label_values(&["get_data"]).inc();
                if error == ZkError::OperationTimeout {
                    ZOO_TIMEOUTS_COUNT.inc();
                }
                error
            });
            timer.observe_duration();
            let node = match result {
                Err(ZkError::NoNode) => continue,
                Err(error) => {
                    let error = Err(error).context(ErrorKind::Backend("node read"));
                    return Some(error.map_err(Error::from));
                },
                Ok((node, _)) => node,
            };
            let node: Result<NodeId> = match serde_json::from_slice(&node) {
                Err(error) => {
                    let error = Err(error).context(ErrorKind::Decode("node info"));
                    error.map_err(Error::from)
                },
                Ok(node) => Ok(node),
            };
            return Some(node);
        }
        None
    }
}

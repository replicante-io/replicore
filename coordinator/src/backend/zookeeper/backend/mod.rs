use std::sync::Arc;

use slog::Logger;

use super::super::super::NodeId;
use super::super::super::Result;
use super::super::super::config::ZookeeperConfig;
use super::super::Backend;
use super::client::Client;

mod cleaner;

use self::cleaner::Cleaner;


/// Zookeeper-backed distributed coordination.
pub struct Zookeeper {
    // Background thread to clean unused nodes.
    _cleaner: Cleaner,
    _client: Arc<Client>,
    node_id: NodeId,
}

impl Zookeeper {
    pub fn new(node_id: NodeId, config: ZookeeperConfig, logger: Logger) -> Result<Zookeeper> {
        let client = Arc::new(Client::new(config.clone(), Some(&node_id), logger.clone())?);
        let cleaner = Cleaner::new(Arc::clone(&client), config, logger)?;
        Ok(Zookeeper {
            _cleaner: cleaner,
            _client: client,
            node_id,
        })
    }
}

impl Backend for Zookeeper {
    fn node_id(&self) -> &NodeId {
        &self.node_id
    }
}

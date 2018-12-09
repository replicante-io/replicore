use slog::Logger;

use super::super::super::Result;
use super::super::super::config::ZookeeperConfig;
use super::super::BackendAdmin;


/// Admin backend for zookeeper distributed coordination.
pub struct ZookeeperAdmin {
    // TODO
}

impl ZookeeperAdmin {
    pub fn new(_config: ZookeeperConfig, _logger: Logger) -> Result<ZookeeperAdmin> {
        Ok(ZookeeperAdmin {})
    }
}

impl BackendAdmin for ZookeeperAdmin {
    // TODO
}

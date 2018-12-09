use std::time::Duration;

use super::NodeId;
use super::Result;


pub mod zookeeper;


/// Distributed coordination backend interface.
pub trait Backend : Send + Sync {
    /// Get the ID of the current node.
    fn node_id(&self) -> &NodeId;

    /// Check if a tombstone exists.
    fn tombstone_check(&self, key: &str) -> Result<Option<NodeId>>;

    /// Ensure a tombstone exists.
    fn tombstone_ensure(&self, key: &str, ttl: Duration) -> Result<NodeId>;
}


/// Distributed coordination admin backend interface.
pub trait BackendAdmin : Send + Sync {
    // TODO
}

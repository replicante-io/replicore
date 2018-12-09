use std::time::Duration;

use super::super::NodeId;
use super::super::Result;
use super::super::backend::Backend;



/// Proxy synchronized access to mock attributes.
pub struct MockBackend {
    pub node_id: NodeId,
}

impl Backend for MockBackend {
    fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    fn tombstone_check(&self, _key: &str) -> Result<Option<NodeId>> {
        panic!("TODO: implement mock tombstone_check")
    }

    fn tombstone_ensure(&self, _key: &str, _ttl: Duration) -> Result<NodeId> {
        panic!("TODO: implement mock tombstone_ensure");
    }
}

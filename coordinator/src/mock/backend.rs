use super::super::NodeId;
use super::super::backend::Backend;



/// Proxy synchronized access to mock attributes.
pub struct MockBackend {
    pub node_id: NodeId,
}

impl Backend for MockBackend {
    fn node_id(&self) -> &NodeId {
        &self.node_id
    }
}

use super::NodeId;
use super::Result;


pub mod zookeeper;


/// Distributed coordination backend interface.
pub trait Backend : Send + Sync {
    /// Get the ID of the current node.
    fn node_id(&self) -> &NodeId;
}


/// Distributed coordination admin backend interface.
pub trait BackendAdmin : Send + Sync {
    /// Iterate over registered nodes.
    fn nodes(&self) -> Nodes;
}


/// Iterator over nodes registered in the coordinator.
pub struct Nodes(Box<dyn Iterator<Item=Result<NodeId>>>);

impl Nodes {
    pub(crate) fn new<I: Iterator<Item=Result<NodeId>> + 'static>(iter: I) -> Nodes {
        Nodes(Box::new(iter))
    }
}

impl Iterator for Nodes {
    type Item = Result<NodeId>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

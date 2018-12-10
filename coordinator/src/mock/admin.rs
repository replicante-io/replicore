use super::super::NodeId;
use super::super::Result;
use super::super::backend::BackendAdmin;
use super::super::backend::Nodes;


/// Proxy synchronized access to mock attributes.
pub struct MockAdmin {}

impl BackendAdmin for MockAdmin {
    fn nodes(&self) -> Nodes {
        Nodes::new(MockNodes {})
    }
}


/// Iterate over nodes in the mock backend.
struct MockNodes {}

impl Iterator for MockNodes {
    type Item = Result<NodeId>;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

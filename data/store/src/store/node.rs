use replicante_data_models::Node as NodeModel;

use super::super::backend::NodeImpl;
use super::super::Result;

/// Operate on the node identified by the provided cluster_id and node_id.
pub struct Node {
    node: NodeImpl,
    attrs: NodeAttribures,
}

impl Node {
    pub(crate) fn new(node: NodeImpl, attrs: NodeAttribures) -> Node {
        Node { node, attrs }
    }

    /// Query the `Node` record, if any is stored.
    pub fn get(&self) -> Result<Option<NodeModel>> {
        self.node.get(&self.attrs)
    }
}

/// Attributes attached to all node operations.
pub struct NodeAttribures {
    pub cluster_id: String,
    pub node_id: String,
}

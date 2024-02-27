use opentracingrust::SpanContext;

use replicante_models_core::agent::Node as NodeModel;

use super::super::backend::NodeImpl;
use super::super::Result;

/// Operate on the node identified by the provided cluster_id and node_id.
pub struct Node {
    node: NodeImpl,
    attrs: NodeAttributes,
}

impl Node {
    pub(crate) fn new(node: NodeImpl, attrs: NodeAttributes) -> Node {
        Node { node, attrs }
    }

    /// Query the `Node` record, if any is stored.
    pub fn get<S>(&self, span: S) -> Result<Option<NodeModel>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.node.get(&self.attrs, span.into())
    }
}

/// Attributes attached to all node operations.
pub struct NodeAttributes {
    pub cluster_id: String,
    pub node_id: String,
}

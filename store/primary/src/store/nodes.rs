use opentracingrust::SpanContext;

use replicante_models_core::agent::Node as NodeModel;

use super::super::backend::NodesImpl;
use super::super::Cursor;
use super::super::Result;

/// Operate on all nodes in the cluster identified by cluster_id.
pub struct Nodes {
    nodes: NodesImpl,
    attrs: NodesAttributes,
}

impl Nodes {
    pub(crate) fn new(nodes: NodesImpl, attrs: NodesAttributes) -> Nodes {
        Nodes { nodes, attrs }
    }

    /// Iterate over nodes in a cluster.
    pub fn iter<S>(&self, span: S) -> Result<Cursor<NodeModel>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.nodes.iter(&self.attrs, span.into())
    }
}

/// Attributes attached to all nodes operations.
pub struct NodesAttributes {
    pub cluster_id: String,
}

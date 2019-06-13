use std::collections::HashSet;

use opentracingrust::SpanContext;

use replicante_models_core::Node as NodeModel;

use super::super::backend::NodesImpl;
use super::super::Cursor;
use super::super::Result;

/// Operate on all nodes in the cluster identified by cluster_id.
pub struct Nodes {
    nodes: NodesImpl,
    attrs: NodesAttribures,
}

impl Nodes {
    pub(crate) fn new(nodes: NodesImpl, attrs: NodesAttribures) -> Nodes {
        Nodes { nodes, attrs }
    }

    /// Iterate over nodes in a cluster.
    pub fn iter<S>(&self, span: S) -> Result<Cursor<NodeModel>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.nodes.iter(&self.attrs, span.into())
    }

    /// Enumerate the different kinds of *active* nodes in the cluster.
    ///
    /// Active nodes are those not stale.
    /// See `Store::cluster::mark_stale` for why nodes are marked stale.
    pub fn kinds<S>(&self, span: S) -> Result<HashSet<String>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.nodes.kinds(&self.attrs, span.into())
    }
}

/// Attributes attached to all nodes operations.
pub struct NodesAttribures {
    pub cluster_id: String,
}

use std::collections::HashSet;

use replicante_data_models::Node as NodeModel;

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
    pub fn iter(&self) -> Result<Cursor<NodeModel>> {
        self.nodes.iter(&self.attrs)
    }

    /// Enumerate the different kinds of *active* nodes in the cluster.
    ///
    /// Active nodes are those not stale.
    /// See `Store::cluster::mark_stale` for why nodes are marked stale.
    pub fn kinds(&self) -> Result<HashSet<String>> {
        self.nodes.kinds(&self.attrs)
    }
}

/// Attributes attached to all nodes operations.
pub struct NodesAttribures {
    pub cluster_id: String,
}

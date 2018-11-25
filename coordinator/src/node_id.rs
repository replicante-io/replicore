use std::collections::BTreeMap;
use std::fmt;

use replicante_util_rndid::RndId;


/// Unique ID for nodes in a Replicante cluster.
///
/// Node IDs are primarily used for debugging and introspection purposes.
/// They are meant to be used as a way to relate events or records back
/// to the node that owns or generate them.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct NodeId {
    extra: BTreeMap<String, String>,
    id: RndId,
}

impl NodeId {
    pub fn new() -> NodeId {
        NodeId {
            extra: BTreeMap::new(),
            id: RndId::new(),
        }
    }

    /// Set the extra attributes attached to this node ID.
    pub fn extra(&mut self, extra: BTreeMap<String, String>) {
        self.extra = extra;
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.id.fmt(f)
    }
}

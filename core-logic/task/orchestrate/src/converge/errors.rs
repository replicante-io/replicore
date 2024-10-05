//! Errors reported by the convergence steps.
use replisdk::core::models::node::Node;

/// The search for node to run cluster.init on returned no target.
#[derive(Debug, thiserror::Error)]
#[error("the search for node to run cluster.init on returned no target")]
pub struct ClusterInitNoTarget;

/// The node has no member address.
#[derive(Debug, thiserror::Error)]
#[error("The node {node_id} has no member address")]
pub struct NodeNoMemberAddress {
    node_id: String,
}

impl From<&Node> for NodeNoMemberAddress {
    fn from(value: &Node) -> Self {
        Self {
            node_id: value.node_id.clone(),
        }
    }
}

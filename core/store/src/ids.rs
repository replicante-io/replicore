//! Reusable containers for resource IDs.
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

/// Information needed to identify node actions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NActionID {
    /// The ID of the action.
    pub action_id: Uuid,

    /// The ID of the cluster the action is for.
    pub cluster_id: String,

    /// The ID of the node the action is targeting.
    pub node_id: String,

    /// The namespace ID of the resource.
    pub ns_id: String,
}

/// Information needed to query namespace scoped resources.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NamespaceID {
    /// ID of the namespace to look into.
    pub id: String,
}

/// Identify a precise namespace scoped resource.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NamespacedResourceID {
    /// The name of the resource.
    pub name: String,

    /// The namespace ID of the resource.
    pub ns_id: String,
}

/// Identify a node and the cluster it is part of.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeID {
    /// The namespace ID the cluster is in.
    pub ns_id: String,

    /// The ID of the cluster the node is part of.
    pub cluster_id: String,

    /// The ID of the node to cancel all actions for.
    pub node_id: String,
}

impl NodeID {
    /// Cancel all actions for a node by its ID.
    pub fn by<S1, S2, S3>(ns_id: S1, cluster_id: S2, node_id: S3) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
    {
        NodeID {
            ns_id: ns_id.into(),
            cluster_id: cluster_id.into(),
            node_id: node_id.into(),
        }
    }
}

/// Information needed to identify orchestrator actions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OActionID {
    /// The ID of the action.
    pub action_id: Uuid,

    /// The ID of the cluster the action is for.
    pub cluster_id: String,

    /// The namespace ID of the resource.
    pub ns_id: String,
}

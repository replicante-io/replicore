//! Errors building a cross-node cluster view.
use uuid::Uuid;

/// Attaching information for cluster to a different cluster's view.
#[derive(Debug, thiserror::Error)]
#[error("attaching information for cluster '{actual_ns}.{actual_cluster}' to '{expect_ns}.{expect_cluster}' cluster's view")]
pub struct ClusterNotMatch {
    // Cluster ID of the information being added.
    pub actual_cluster: String,

    // Namespace ID of the information being added.
    pub actual_ns: String,

    // Cluster ID of the view under construction.
    pub expect_cluster: String,

    // Namespace ID of the view under construction.
    pub expect_ns: String,
}

/// Can't add finished node action to view for cluster.
#[derive(Debug, thiserror::Error)]
#[error("can't add finished node action '{action_id}' for node '{node_id}' to view for cluster '{ns_id}.{cluster_id}'")]
pub struct FinishedNAction {
    // Namespace ID the cluster is in.
    pub ns_id: String,

    // ID of the cluster the view is for.
    pub cluster_id: String,

    // ID of the node the action is for.
    pub node_id: String,

    // ID of the finished node action being added.
    pub action_id: Uuid,
}

/// Can't add finished orchestrator action to view for cluster.
#[derive(Debug, thiserror::Error)]
#[error("can't add finished orchestrator action '{action_id}' to view for cluster '{ns_id}.{cluster_id}'")]
pub struct FinishedOAction {
    // Namespace ID the cluster is in.
    pub ns_id: String,

    // ID of the cluster the view is for.
    pub cluster_id: String,

    // ID of the finished orchestrator action being added.
    pub action_id: Uuid,
}

/// Can't remove unfinished node action to view for cluster.
#[derive(Debug, thiserror::Error)]
#[error("can't remove unfinished node action '{action_id}' for node '{node_id}' to view for cluster '{ns_id}.{cluster_id}'")]
pub struct UnfinishedNAction {
    // Namespace ID the cluster is in.
    pub ns_id: String,

    // ID of the cluster the view is for.
    pub cluster_id: String,

    // ID of the node the action is for.
    pub node_id: String,

    // ID of the finished node action being added.
    pub action_id: Uuid,
}

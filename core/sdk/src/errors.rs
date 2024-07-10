//! Errors reported by the Control Plane SDK.
use uuid::Uuid;

/// Node Action already exists for cluster.
#[derive(Debug, thiserror::Error)]
#[error("Node Action '{action_id}' already exists for cluster '{ns_id}.{cluster_id}' on node '{node_id}")]
pub struct NActionExists {
    pub ns_id: String,
    pub cluster_id: String,
    pub node_id: String,
    pub action_id: Uuid,
}

/// Orchestrator Action already exists for cluster.
#[derive(Debug, thiserror::Error)]
#[error("Orchestrator Action '{action_id}' already exists for cluster '{ns_id}.{cluster_id}'")]
pub struct OActionExists {
    pub ns_id: String,
    pub cluster_id: String,
    pub action_id: Uuid,
}

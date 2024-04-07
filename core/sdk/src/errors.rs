//! Errors reported by the Control Plane SDK.
use uuid::Uuid;

/// Orchestrator Action already exists for cluster.
#[derive(Debug, thiserror::Error)]
#[error("Orchestrator Action '{action_id}' already exists for cluster '{ns_id}.{cluster_id}'")]
pub struct ActionExists {
    pub ns_id: String,
    pub cluster_id: String,
    pub action_id: Uuid,
}

//! Reusable containers for resource IDs.
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

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

/// Information needed to identifier orchestrator actions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OActionID {
    /// The ID of the action.
    pub action_id: Uuid,

    /// The ID of the cluster the action is fore.
    pub cluster_id: String,

    /// The namespace ID of the resource.
    pub ns_id: String,
}

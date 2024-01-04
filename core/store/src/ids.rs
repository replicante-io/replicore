//! Reusable containers for resource IDs.
use serde::Deserialize;
use serde::Serialize;

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

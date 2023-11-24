//! Reusable containers for resource IDs.
/// Information needed to query namespace scoped resources.
#[derive(Clone, Debug)]
pub struct NamespaceID {
    /// ID of the namespace to look into.
    pub id: String,
}

/// Identify a precise namespace scoped resource.
#[derive(Clone, Debug)]
pub struct NamespacedResourceID {
    /// The name of the resource.
    pub name: String,

    /// The namespace ID of the resource.
    pub ns_id: String,
}

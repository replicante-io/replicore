//! Common errors from and for RepliCore implementation

/// The cluster is not in the active status.
#[derive(Debug, thiserror::Error)]
#[error("the cluster '{ns_id}.{cluster_id}' is not in the active status")]
pub struct ClusterNotActive {
    pub cluster_id: String,
    pub ns_id: String,
}

impl ClusterNotActive {
    /// The cluster is not in the active status.
    pub fn new<S1: Into<String>, S2: Into<String>>(ns_id: S1, cluster_id: S2) -> Self {
        Self {
            cluster_id: cluster_id.into(),
            ns_id: ns_id.into(),
        }
    }
}

/// The expected cluster specification was not found.
#[derive(Debug, thiserror::Error)]
#[error("the expected cluster specification '{ns_id}.{cluster_id}' was not found")]
pub struct ClusterNotFound {
    pub cluster_id: String,
    pub ns_id: String,
}

impl ClusterNotFound {
    /// The expected cluster specification was not found.
    pub fn new<S1: Into<String>, S2: Into<String>>(ns_id: S1, cluster_id: S2) -> Self {
        Self {
            cluster_id: cluster_id.into(),
            ns_id: ns_id.into(),
        }
    }
}

/// The namespace is not in the active status.
#[derive(Debug, thiserror::Error)]
#[error("the namespace '{ns_id}' is not in the active status")]
pub struct NamespaceNotActive {
    pub ns_id: String,
}

impl NamespaceNotActive {
    /// The namespace is not in the active status.
    pub fn new<S: Into<String>>(ns_id: S) -> Self {
        Self {
            ns_id: ns_id.into(),
        }
    }
}

/// The expected namespace was not found.
#[derive(Debug, thiserror::Error)]
#[error("the expected namespace '{ns_id}' was not found")]
pub struct NamespaceNotFound {
    pub ns_id: String,
}

impl NamespaceNotFound {
    /// The expected namespace was not found.
    pub fn new<S: Into<String>>(ns_id: S) -> Self {
        Self {
            ns_id: ns_id.into(),
        }
    }
}

//! Errors building a cross-node cluster view.

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

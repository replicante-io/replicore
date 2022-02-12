use thiserror::Error;

/// The attempt to build a ClusterView resulted in a corrupt or invalid view.
#[derive(Error, Debug)]
pub enum ClusterViewCorrupt {
    #[error("cannot update view of cluster in namespace {0} with a record from namespace {1}")]
    NamespaceClash(String, String),

    #[error("cannot update view of cluster ID {0}.{1} with a record from cluster ID {0}.{2}")]
    ClusterIdClash(String, String, String),
}

impl ClusterViewCorrupt {
    /// Adding a record that belongs to a different namespace.
    pub fn namespace_clash<S1, S2>(expected_ns: S1, found_ns: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let expected_ns = expected_ns.into();
        let found_ns = found_ns.into();
        ClusterViewCorrupt::NamespaceClash(expected_ns, found_ns)
    }

    /// Adding a record that does not belong to the cluster.
    pub fn cluster_id_clash<S1, S2, S3>(
        namespace: S1,
        expected_cluster: S2,
        found_cluster: S3,
    ) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
    {
        let namespace = namespace.into();
        let expected_cluster = expected_cluster.into();
        let found_cluster = found_cluster.into();
        ClusterViewCorrupt::ClusterIdClash(namespace, expected_cluster, found_cluster)
    }
}

use thiserror::Error;

/// The attempt to build a ClusterView resulted in a corrupt or invalid view.
#[derive(Error, Debug)]
pub enum ClusterViewCorrupt {
    #[error("cannot update view of cluster ID {0}.{1} with a record from cluster ID {0}.{2}")]
    // namespace, expect, actual
    ClusterIdClash(String, String, String),

    #[error("view of cluster {0}.{1} already contains an agent with ID {2}")]
    // namespace, cluster_id, agent_id.
    DuplicateAgent(String, String, String),

    #[error("view of cluster {0}.{1} already contains a agent info for agent with ID {2}")]
    // namespace, cluster_id, agent_id.
    DuplicateAgentInfo(String, String, String),

    #[error("view of cluster {0}.{1} already contains a node with ID {2}")]
    // namespace, cluster_id, node_id.
    DuplicateNode(String, String, String),

    #[error("view of cluster {0}.{1} already contains shard ID {3} on node ID {2}")]
    // namespace, cluster_id, node_id, shard_id.
    DuplicateShard(String, String, String, String),

    #[error("cannot update view of cluster in namespace {0} with a record from namespace {1}")]
    // expected, actual
    NamespaceClash(String, String),
}

impl ClusterViewCorrupt {
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

    /// Adding the same agent twice.
    pub fn duplicate_agent<S1, S2, S3>(namespace: S1, cluster_id: S2, agent_id: S3) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
    {
        let namespace = namespace.into();
        let cluster_id = cluster_id.into();
        let agent_id = agent_id.into();
        ClusterViewCorrupt::DuplicateAgent(namespace, cluster_id, agent_id)
    }

    /// Adding the same agent info twice.
    pub fn duplicate_agent_info<S1, S2, S3>(namespace: S1, cluster_id: S2, agent_id: S3) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
    {
        let namespace = namespace.into();
        let cluster_id = cluster_id.into();
        let agent_id = agent_id.into();
        ClusterViewCorrupt::DuplicateAgentInfo(namespace, cluster_id, agent_id)
    }

    /// Adding the same node twice.
    pub fn duplicate_node<S1, S2, S3>(namespace: S1, cluster_id: S2, node_id: S3) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
    {
        let namespace = namespace.into();
        let cluster_id = cluster_id.into();
        let node_id = node_id.into();
        ClusterViewCorrupt::DuplicateNode(namespace, cluster_id, node_id)
    }

    /// Adding the same shard twice on one node.
    pub fn duplicate_shard<S1, S2, S3, S4>(
        namespace: S1,
        cluster_id: S2,
        node_id: S3,
        shard_id: S4,
    ) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
        S4: Into<String>,
    {
        let namespace = namespace.into();
        let cluster_id = cluster_id.into();
        let node_id = node_id.into();
        let shard_id = shard_id.into();
        ClusterViewCorrupt::DuplicateShard(namespace, cluster_id, node_id, shard_id)
    }

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
}

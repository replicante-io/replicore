use replicante_data_models::ClusterDiscovery;

use super::super::backend::ClusterImpl;
use super::super::Result;

/// Operate on cluster-level models.
pub struct Cluster {
    cluster: ClusterImpl,
    attrs: ClusterAttribures,
}

impl Cluster {
    pub(crate) fn new(cluster: ClusterImpl, attrs: ClusterAttribures) -> Cluster {
        Cluster { cluster, attrs }
    }

    /// Query a `ClusterDiscovery` record, if any is stored.
    pub fn discovery(&self) -> Result<Option<ClusterDiscovery>> {
        self.cluster.discovery(&self.attrs)
    }
}

/// Attributes attached to all cluster-level operations.
pub struct ClusterAttribures {
    pub cluster_id: String,
}

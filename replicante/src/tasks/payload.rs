use replicante_data_models::ClusterDiscovery;

/// Cluster refresh task parameters.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ClusterRefreshPayload {
    pub cluster: ClusterDiscovery,
    pub snapshot: bool,
}

impl ClusterRefreshPayload {
    pub fn new(cluster: ClusterDiscovery, snapshot: bool) -> ClusterRefreshPayload {
        ClusterRefreshPayload { cluster, snapshot }
    }
}

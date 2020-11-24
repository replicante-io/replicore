use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Cluster orchestration task parameters.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct OrchestrateClusterPayload {
    pub cluster_id: String,
    pub namespace: String,
}

impl OrchestrateClusterPayload {
    pub fn new<S1, S2>(namespace: S1, cluster_id: S2) -> OrchestrateClusterPayload
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let cluster_id = cluster_id.into();
        let namespace = namespace.into();
        OrchestrateClusterPayload {
            cluster_id,
            namespace,
        }
    }
}

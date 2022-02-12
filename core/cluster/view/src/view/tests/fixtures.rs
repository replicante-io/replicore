use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::ClusterSettings;

/// Fixtures for a fixtional MongoDB cluster.
pub mod cluster_mongodb {
    use super::*;

    pub const CLUSTER_ID: &str = "colours";
    pub const NAMESPACE: &str = "default";

    pub fn discovery() -> ClusterDiscovery {
        ClusterDiscovery::new(CLUSTER_ID, vec![])
    }
}

/// Fixtures for a fixtional Zookeeepr cluster.
pub mod cluster_zookeeper {
    use super::*;

    pub const CLUSTER_ID: &str = "animals";
    pub const NAMESPACE: &str = "default";

    pub fn settings() -> ClusterSettings {
        ClusterSettings::synthetic(NAMESPACE, CLUSTER_ID)
    }
}

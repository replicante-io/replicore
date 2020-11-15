use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_models_core::cluster::discovery::DiscoverySettings;

/// Clusters discovery task parameters.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct DiscoverClustersPayload {
    pub settings: DiscoverySettings,
}

impl DiscoverClustersPayload {
    pub fn new(settings: DiscoverySettings) -> DiscoverClustersPayload {
        DiscoverClustersPayload { settings }
    }
}

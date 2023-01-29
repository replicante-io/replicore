use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_models_core::cluster::discovery::DiscoverySettings;
use replisdk::core::models::platform::Platform;

/// Clusters discovery task parameters.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct DiscoverClustersPayload {
    pub settings: Option<DiscoverySettings>,
    pub platform: Option<Platform>,
}

impl DiscoverClustersPayload {
    pub fn new(platform: Platform) -> DiscoverClustersPayload {
        DiscoverClustersPayload {
            settings: None,
            platform: Some(platform),
        }
    }

    pub fn new_discovery(settings: DiscoverySettings) -> DiscoverClustersPayload {
        DiscoverClustersPayload {
            settings: Some(settings),
            platform: None,
        }
    }
}

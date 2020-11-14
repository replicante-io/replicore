use serde_derive::Deserialize;
use serde_derive::Serialize;

use crate::cluster::discovery::DiscoveryBackend;

/// Cluster discovery settings for a single discovery backend.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct DiscoverySettings {
    /// Backend to discover clusters from.
    #[serde(flatten)]
    pub backend: DiscoveryBackend,

    /// Enable or disable discovery against this backend.
    #[serde(default = "DiscoverySettings::default_enabled")]
    pub enabled: bool,

    /// Interval, in seconds, between discovery runs.
    pub interval: i64,
}

impl DiscoverySettings {
    fn default_enabled() -> bool {
        true
    }
}

use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_models_core::cluster::discovery::HttpDiscovery;

/// Agent discovery configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub http: Vec<HttpDiscovery>,
}

impl Default for Config {
    fn default() -> Config {
        Config { http: Vec::new() }
    }
}

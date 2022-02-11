use std::collections::BTreeMap;

use serde_derive::Deserialize;
use serde_derive::Serialize;

mod zookeeper;

pub use self::zookeeper::ZookeeperConfig;

/// Backend specific configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "backend", content = "options", deny_unknown_fields)]
pub enum Backend {
    /// Use zookeeper as a coordination system (recommended, default).
    #[serde(rename = "zookeeper")]
    Zookeeper(ZookeeperConfig),
}

impl Default for Backend {
    fn default() -> Backend {
        Backend::Zookeeper(ZookeeperConfig::default())
    }
}

/// Distributed coordinator configuration options.
#[derive(Clone, Default, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default, flatten)]
    pub backend: Backend,

    /// User specified key/value map attached to node IDs.
    ///
    /// This data is not used by the system and is provided to help users debug
    /// and otherwise label nodes for whatever needs they may have.
    #[serde(default)]
    pub node_attributes: BTreeMap<String, String>,
}

use std::collections::BTreeMap;


/// Distributed coordinator configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    /// User specified key/value map attached to node IDs.
    ///
    /// This data is not used by the system and is provided to help users debug
    /// and otherwise label nodes for whatever needs they may have.
    #[serde(default)]
    pub node_attributes: BTreeMap<String, String>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            node_attributes: BTreeMap::new(),
        }
    }
}

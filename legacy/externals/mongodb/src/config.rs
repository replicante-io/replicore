use serde::Deserialize;
use serde::Serialize;

/// MongoDB storage configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct CommonConfig {
    /// URI of the MongoDB Replica Set or sharded cluster to connect to.
    pub uri: String,
}

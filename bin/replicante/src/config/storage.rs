use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_store_primary::Config as PrimaryStoreConfig;

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Primary store configuration.
    pub primary: PrimaryStoreConfig,
}

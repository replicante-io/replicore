use serde::Deserialize;
use serde::Serialize;

use replicante_store_primary::Config as PrimaryStoreConfig;
use replicante_store_view::Config as ViewStoreConfig;

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Primary store configuration.
    pub primary: PrimaryStoreConfig,

    /// View store configuration.
    pub view: ViewStoreConfig,
}

use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Events streaming backend configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "backend", content = "options")]
pub enum Config {
    /// Wrap the configured store for use as the event stream.
    #[serde(rename = "store")]
    Store,
}

impl Default for Config {
    fn default() -> Config {
        Config::Store
    }
}

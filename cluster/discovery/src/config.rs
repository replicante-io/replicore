use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Agent discovery configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub files: Vec<String>,
}

impl Default for Config {
    fn default() -> Config {
        Config { files: Vec::new() }
    }
}

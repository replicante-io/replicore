use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde_yaml;

use replicante_data_store::Config as StorageConfig;
use replicante_logging::Config as LoggingConfig;
use replicante_util_tracing::Config as TracingConfig;

use super::Result;

use super::components::discovery::Config as DiscoveryConfig;
use super::interfaces::api::Config as APIConfig;


/// Replicante configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// API server configuration.
    #[serde(default)]
    pub api: APIConfig,

    /// Agent discovery configuration.
    #[serde(default)]
    pub discovery: DiscoveryConfig,

    /// Logging configuration.
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Storage layer configuration.
    #[serde(default)]
    pub storage: StorageConfig,

    /// Distributed tracing configuration.
    #[serde(default)]
    pub tracing: TracingConfig,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            api: APIConfig::default(),
            discovery: DiscoveryConfig::default(),
            logging: LoggingConfig::default(),
            storage: StorageConfig::default(),
            tracing: TracingConfig::default(),
        }
    }
}

impl Config {
    /// Loads the configuration from the given [`std::fs::File`].
    ///
    /// [`std::fs::File`]: https://doc.rust-lang.org/std/fs/struct.File.html
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let config = File::open(path)?;
        Config::from_reader(config)
    }

    /// Loads the configuration from the given [`std::io::Read`].
    ///
    /// [`std::io::Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
    pub fn from_reader<R: Read>(reader: R) -> Result<Config> {
        let conf = serde_yaml::from_reader(reader)?;
        Ok(conf)
    }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::Config;

    #[test]
    #[should_panic(expected = "invalid type: string")]
    fn from_reader_error() {
        let cursor = Cursor::new("some other text");
        Config::from_reader(cursor).unwrap();
    }

    #[test]
    fn from_reader_ok() {
        let cursor = Cursor::new("{}");
        Config::from_reader(cursor).unwrap();
    }
}

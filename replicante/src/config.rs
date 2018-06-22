use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde_yaml;

use replicante_data_store::Config as StorageConfig;
use replicante_util_tracing::Config as TracingConfig;

use super::Result;

use super::components::discovery::Config as DiscoveryConfig;
use super::interfaces::api::Config as APIConfig;
use super::logging::Config as LoggingConfig;


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
    use std::path::Path;

    use super::super::Error;
    use super::super::ErrorKind;
    use super::Config;

    /// Helper function to find fixtrue files.
    ///
    /// Neede because `cargo test` and `cargo kcov` behave differently with regards to the working
    /// directory of the executed command (`cargo test` moves to the crate, `cargo kcov` does not).
    fn fixture_path(path: &str) -> String {
        let nested = format!("replicante/{}", path);
        if Path::new(&nested).exists() {
            nested
        } else {
            path.to_string()
        }
    }

    #[test]
    fn from_reader_error() {
        let cursor = Cursor::new("some other text");
        match Config::from_reader(cursor) {
            Err(Error(ErrorKind::YamlDecode(_), _)) => (),
            Err(err) => panic!("Unexpected error: {:?}", err),
            Ok(_) => panic!("Unexpected success!"),
        };
    }

    #[test]
    fn from_reader_ok() {
        let cursor = Cursor::new("{}");
        Config::from_reader(cursor).unwrap();
    }

    #[test]
    // NOTE: this cannot validate missing attributes.
    fn ensure_example_config_matches_default() {
        let default = Config::default();
        let example = Config::from_file(fixture_path("../replicante.example.yaml"))
            .expect("Cannot open example configuration");
        assert_eq!(
            default, example,
            "Default configuration does not match replicante.example.yaml"
        );
    }
}

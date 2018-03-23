use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde_yaml;

use super::Result;
use super::logging::Config as LoggingConfig;


/// Replicante configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Logging configuration.
    #[serde(default)]
    pub logging: LoggingConfig,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            logging: LoggingConfig::default(),
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

    use super::super::Error;
    use super::super::ErrorKind;
    use super::Config;

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
}

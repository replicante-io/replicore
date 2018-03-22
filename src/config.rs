use std::io::Read;
use serde_yaml;

use super::Result;


#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
}

impl Default for Config {
    fn default() -> Config {
        Config {}
    }
}

impl Config {
    /// Loads the configuration from the given [`std::io::Read`].
    ///
    /// [`std::io::Read`]: https://doc.rust-lang.org/nightly/std/io/trait.Read.html
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

    // TODO: test default
}

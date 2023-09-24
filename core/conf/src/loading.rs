//! Load configuration from files.
use std::fs::File;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;

use crate::Conf;

/// Errors handling Replicante Core configuration.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Unable to decode configuration from file at the given path.
    #[error("unable to decode configuration from file at '{0}'")]
    // (path,)
    Decode(String),

    /// Unable to read configuration file at the given path.
    #[error("unable to read configuration file at '{0}'")]
    // (path,)
    Open(String),

    /// Configuration file not found at the given path.
    #[error("configuration file not found at '{0}'")]
    // (path,)
    PathNotFound(String),
}

/// Load process configuration from the specified path.
pub fn load(path: &str) -> Result<Conf> {
    // Check if the configuration file exists and return the default if it does not.
    if !PathBuf::from(path).exists() {
        let error = Error::PathNotFound(path.to_string());
        let error = anyhow::anyhow!(error);
        return Err(error);
    }

    // Load and deserialize the agent configuration.
    let file = File::open(path).with_context(|| Error::Open(path.into()))?;
    let conf = serde_yaml::from_reader(file).with_context(|| Error::Decode(path.into()))?;
    Ok(conf)
}

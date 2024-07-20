//! Configuration for the SQLite background tasks backend.
use serde::Deserialize;
use serde::Serialize;

/// SQLite specific configuration for the background tasks interface.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Conf {
    /// Path to the SQLite DB file.
    pub path: String,

    /// Delay between DB queries for pending/retry tasks to become available for execution.
    #[serde(default = "Conf::default_poll_delay_s")]
    pub poll_delay_s: u64,
}

impl Conf {
    fn default_poll_delay_s() -> u64 {
        30
    }

    /// Initialise tasks configuration with a SQLite path and defaults.
    pub fn new<S>(path: S) -> Conf
    where
        S: Into<String>,
    {
        Conf {
            path: path.into(),
            poll_delay_s: Conf::default_poll_delay_s(),
        }
    }
}

/// The SQLite background tasks backend configuration is not valid.
#[derive(Debug, thiserror::Error)]
#[error("the SQLite background tasks backend configuration is not valid")]
pub struct ConfError;

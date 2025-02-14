//! Configuration for the SQLite events backend.
use serde::Deserialize;
use serde::Serialize;

/// SQLite specific configuration for the events interface.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Conf {
    /// Path to the SQLite DB file.
    pub path: String,

    /// Events retention and history clean up rules.
    #[serde(default)]
    pub retention: Retention,
}

/// Events retention and history clean up rules.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Retention {
    /// Number of days to keep events in the DB for.
    #[serde(default = "Retention::default_age")]
    pub age: u64,
}

impl Default for Retention {
    fn default() -> Self {
        Retention {
            age: Self::default_age(),
        }
    }
}

impl Retention {
    fn default_age() -> u64 {
        30
    }
}

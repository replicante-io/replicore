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
    pub age: u32,

    /// Maximum number of expired events to delete in a single history clean loop.
    #[serde(default = "Retention::default_clean_batch")]
    pub clean_batch: u32,

    /// Minutes to wait between each run of the history clean loop.
    #[serde(default = "Retention::default_delay")]
    pub clean_delay: u32,
}

impl Default for Retention {
    fn default() -> Self {
        Retention {
            age: Self::default_age(),
            clean_batch: Self::default_clean_batch(),
            clean_delay: Self::default_delay(),
        }
    }
}

impl Retention {
    fn default_age() -> u32 {
        30
    }

    fn default_clean_batch() -> u32 {
        500
    }

    fn default_delay() -> u32 {
        1
    }
}

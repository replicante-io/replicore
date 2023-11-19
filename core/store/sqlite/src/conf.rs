//! Configuration for the SQLite persistent store backend.
use serde::Deserialize;
use serde::Serialize;

/// SQLite specific configuration for the persistent store interface.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Conf {
    /// Path to the SQLite DB file.
    pub path: String,
}

/// The SQLite persistent store backend configuration is not valid.
#[derive(Debug, thiserror::Error)]
#[error("the SQLite persistent store backend configuration is not valid")]
pub struct ConfError;

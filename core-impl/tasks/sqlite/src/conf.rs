//! Configuration for the SQLite background tasks backend.
use serde::Deserialize;
use serde::Serialize;

/// SQLite specific configuration for the background tasks interface.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Conf {
    /// Path to the SQLite DB file.
    pub path: String,
}

/// The SQLite background tasks backend configuration is not valid.
#[derive(Debug, thiserror::Error)]
#[error("the SQLite background tasks backend configuration is not valid")]
pub struct ConfError;

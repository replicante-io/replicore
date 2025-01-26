//! Configuration for the SQLite coordinator backend.
use serde::Deserialize;
use serde::Serialize;

/// SQLite specific configuration for the coordinator interface.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Conf {
    /// Path to the SQLite DB file.
    pub path: String,

    /// Time to live, in seconds, before leases are either renewed or lost.
    ///
    /// To reduce the likelihood of multiple [`Lease`] objects thinking they hold the lease,
    /// renewals MUST be performed before the TTL expires.
    /// Details vary based on the exact implementation, check each backend documentation if needed.
    #[serde(default = "Conf::default_ttl")]
    pub ttl: u64,

    /// Interval, in seconds, between DB checks and renewal attempts.
    ///
    /// Must be less or equal to half the TTL value.
    #[serde(default = "Conf::default_watch_interval")]
    pub watch_interval: u64,
}

impl Conf {
    fn default_ttl() -> u64 {
        60
    }

    fn default_watch_interval() -> u64 {
        30
    }
}

/// The SQLite persistent store backend configuration is not valid.
#[derive(Debug, thiserror::Error)]
#[error("the SQLite persistent store backend configuration is not valid")]
pub struct ConfError;

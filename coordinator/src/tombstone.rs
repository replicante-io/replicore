use std::sync::Arc;
use std::time::Duration;

use super::NodeId;
use super::Result;
use super::backend::Backend;


/// Marker that automatically clears itself after a TTL.
///
/// Tombstones can be used to "remember" the occurrence of an event for the given time.
///
/// For example, cluster refresh tasks create a tombstone when done and are skipped if
/// the tombstone exists to prevent overloading cluster nodes with requests.
pub struct Tombstone {
    backend: Arc<Backend>,
    key: String,
}

impl Tombstone {
    pub(crate) fn new(backend: Arc<Backend>, key: String) -> Tombstone {
        Tombstone {
            backend,
            key,
        }
    }

    /// Check if the tombstone exists.
    pub fn check(&self) -> Result<Option<NodeId>> {
        self.backend.tombstone_check(&self.key)
    }

    /// Ensure the tombstone exists.
    pub fn ensure(&self, ttl: Duration) -> Result<NodeId> {
        self.backend.tombstone_ensure(&self.key, ttl)
    }
}

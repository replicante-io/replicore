//! Cluster discovery related events.
use serde::Deserialize;
use serde::Serialize;

use replisdk::core::models::cluster::ClusterDiscovery;

/// Event code emitted when a ClusterDiscovery for a new cluster is found.
pub const EVENT_NEW: &str = "CLUSTER_DISCOVERY_NEW";

/// Event code emitted when a synthetic ClusterSpec is created.
pub const EVENT_SYNTHETIC: &str = "CLUSTER_DISCOVERY_SYNTHETIC";

/// Event code emitted when a ClusterDiscovery for a cluster is updated.
pub const EVENT_UPDATE: &str = "CLUSTER_DISCOVERY_UPDATE";

/// Payload for [`ClusterDiscovery`] update events.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UpdatePayload {
    /// The [`ClusterDiscovery`] record before the update.
    pub before: ClusterDiscovery,

    /// The [`ClusterDiscovery`] record after the update.
    pub after: ClusterDiscovery,
}

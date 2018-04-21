use super::ClusterDiscovery;


/// Enumerates all possible events emitted by the system.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload", deny_unknown_fields)]
pub enum Events {
    /// The service discovery found a new cluster.
    ClusterNew(ClusterDiscovery),
}

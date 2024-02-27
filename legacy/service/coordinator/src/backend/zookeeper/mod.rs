use serde::Deserialize;
use serde::Serialize;

use super::super::NodeId;

mod admin;
mod client;
mod constants;
mod coordinator;
mod metrics;

pub use self::admin::ZookeeperAdmin;
pub use self::coordinator::Zookeeper;
pub use self::metrics::register_metrics;

/// Election candidate payload stored in the election candidate znodes.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
struct ElectionCandidateInfo {
    pub owner: NodeId,
}

/// Election payload stored in the election znodes.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
struct ElectionInfo {
    pub name: String,
}

/// Non-blocking locks payload stored in the ephimeral nodes.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
struct NBLockInfo {
    pub name: String,
    pub owner: NodeId,
}

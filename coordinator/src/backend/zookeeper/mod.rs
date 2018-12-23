use super::super::NodeId;

mod admin;
mod client;
mod coordinator;
mod metrics;

pub use self::admin::ZookeeperAdmin;
pub use self::coordinator::Zookeeper;
pub use self::metrics::register_metrics;


/// Non-blocking locks payload stored in the ephimeral nodes.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
struct NBLockInfo {
    pub name: String,
    pub owner: NodeId,
}

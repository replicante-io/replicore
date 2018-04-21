extern crate serde;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate serde_json;

extern crate replicante_agent_models;


mod agent;
mod cluster;
mod datastore;
mod events;

pub use self::agent::Agent;
pub use self::agent::AgentInfo;
pub use self::agent::AgentStatus;

pub use self::cluster::ClusterDiscovery;
pub use self::cluster::ClusterMeta;

pub use self::datastore::Node;
pub use self::datastore::Shard;
pub use self::datastore::ShardRole;

pub use self::events::Events;

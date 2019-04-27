//! System events models and attributes.
use chrono::DateTime;
use chrono::Utc;

use super::Agent;
use super::AgentInfo;
use super::AgentStatus;
use super::ClusterDiscovery;
use super::Node;
use super::Shard;

mod builder;

use self::builder::EventBuilder;

/// Metadata attached to agent new events.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct AgentNew {
    pub cluster_id: String,
    pub host: String,
}

/// Metadata attached to agent info changed.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct AgentInfoChanged {
    pub after: AgentInfo,
    pub before: AgentInfo,
    pub cluster_id: String,
}

/// Metadata attached to agent status change events.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct AgentStatusChange {
    pub after: AgentStatus,
    pub before: AgentStatus,
    pub cluster_id: String,
    pub host: String,
}

/// Metadata attached to cluster status change events.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ClusterChanged {
    pub after: ClusterDiscovery,
    pub before: ClusterDiscovery,
    pub cluster_id: String,
}

/// Metadata attached to node changed events.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct NodeChanged {
    pub after: Node,
    pub before: Node,
    pub cluster_id: String,
    pub node_id: String,
}

/// Metadata attached to shard allocation changed events.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ShardAllocationChanged {
    pub after: Shard,
    pub before: Shard,
    pub cluster_id: String,
    pub shard_id: String,
    pub node_id: String,
}

/// Enumerates all possible events emitted by the system.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum EventPayload {
    /// An agent was found to be down.
    #[serde(rename = "AGENT_DOWN")]
    AgentDown(AgentStatusChange),

    /// Information about an agent changed.
    #[serde(rename = "AGENT_INFO_CHANGED")]
    AgentInfoChanged(Box<AgentInfoChanged>),

    /// Information about an agent was collected for the first time.
    #[serde(rename = "AGENT_INFO_NEW")]
    AgentInfoNew(AgentInfo),

    /// An agent was discovered for the first time.
    #[serde(rename = "AGENT_NEW")]
    AgentNew(AgentNew),

    /// An agent was found to be up.
    #[serde(rename = "AGENT_UP")]
    AgentUp(AgentStatusChange),

    /// Service discovery record for a cluster changed.
    #[serde(rename = "CLUSTER_CHANGED")]
    ClusterChanged(ClusterChanged),

    /// Service discovery found a new cluster.
    #[serde(rename = "CLUSTER_NEW")]
    ClusterNew(ClusterDiscovery),

    /// A datastore node has changed.
    #[serde(rename = "NODE_CHANGED")]
    NodeChanged(Box<NodeChanged>),

    /// A datastore node was found to be down.
    #[serde(rename = "NODE_DOWN")]
    NodeDown(AgentStatusChange),

    /// A datastore node was found for the first time.
    #[serde(rename = "NODE_NEW")]
    NodeNew(Node),

    /// A datastore node was found to be up.
    #[serde(rename = "NODE_UP")]
    NodeUp(AgentStatusChange),

    /// A shard on a node has changed.
    #[serde(rename = "SHARD_ALLOCATION_CHANGED")]
    ShardAllocationChanged(Box<ShardAllocationChanged>),

    /// A shard was found for the first time on a node.
    #[serde(rename = "SHARD_ALLOCATION_NEW")]
    ShardAllocationNew(Shard),

    /// Snapshot state of an agent status.
    #[serde(rename = "SNAPSHOT_AGENT")]
    SnapshotAgent(Agent),

    /// Snapshot state of an agent info.
    #[serde(rename = "SNAPSHOT_AGENT_INFO")]
    SnapshotAgentInfo(AgentInfo),

    /// Snapshot state of a cluster discovery.
    #[serde(rename = "SNAPSHOT_DISCOVERY")]
    SnapshotDiscovery(ClusterDiscovery),

    /// Snapshot state of a cluster node.
    #[serde(rename = "SNAPSHOT_NODE")]
    SnapshotNode(Node),

    /// Snapshot state of a cluster shard.
    #[serde(rename = "SNAPSHOT_SHARD")]
    SnapshotShard(Shard),
}

/// Model an event that is emitted by the system.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Event {
    #[serde(flatten)]
    pub payload: EventPayload,
    pub timestamp: DateTime<Utc>,
}

impl Event {
    /// Create an helper object to build `Event`s.
    pub fn builder() -> EventBuilder {
        EventBuilder::new()
    }

    /// Look up the cluster ID for the event, if they have one.
    pub fn cluster_id(&self) -> Option<&str> {
        match self.payload {
            EventPayload::AgentDown(ref data) => Some(&data.cluster_id),
            EventPayload::AgentInfoChanged(ref data) => Some(&data.cluster_id),
            EventPayload::AgentInfoNew(ref data) => Some(&data.cluster_id),
            EventPayload::AgentNew(ref data) => Some(&data.cluster_id),
            EventPayload::AgentUp(ref data) => Some(&data.cluster_id),
            EventPayload::ClusterChanged(ref data) => Some(&data.cluster_id),
            EventPayload::ClusterNew(ref data) => Some(&data.cluster_id),
            EventPayload::NodeChanged(ref data) => Some(&data.cluster_id),
            EventPayload::NodeDown(ref data) => Some(&data.cluster_id),
            EventPayload::NodeNew(ref data) => Some(&data.cluster_id),
            EventPayload::NodeUp(ref data) => Some(&data.cluster_id),
            EventPayload::ShardAllocationChanged(ref data) => Some(&data.cluster_id),
            EventPayload::ShardAllocationNew(ref data) => Some(&data.cluster_id),
            EventPayload::SnapshotAgent(ref data) => Some(&data.cluster_id),
            EventPayload::SnapshotAgentInfo(ref data) => Some(&data.cluster_id),
            EventPayload::SnapshotDiscovery(ref data) => Some(&data.cluster_id),
            EventPayload::SnapshotNode(ref data) => Some(&data.cluster_id),
            EventPayload::SnapshotShard(ref data) => Some(&data.cluster_id),
        }
    }

    /// Returns the event "code", the string that represents the event type.
    pub fn code(&self) -> &'static str {
        match self.payload {
            EventPayload::AgentDown(_) => "AGENT_DOWN",
            EventPayload::AgentInfoChanged(_) => "AGENT_INFO_CHANGED",
            EventPayload::AgentInfoNew(_) => "AGENT_INFO_NEW",
            EventPayload::AgentNew(_) => "AGENT_NEW",
            EventPayload::AgentUp(_) => "AGENT_UP",
            EventPayload::ClusterChanged(_) => "CLUSTER_CHANGED",
            EventPayload::ClusterNew(_) => "CLUSTER_NEW",
            EventPayload::NodeChanged(_) => "NODE_CHANGED",
            EventPayload::NodeDown(_) => "NODE_DOWN",
            EventPayload::NodeNew(_) => "NODE_NEW",
            EventPayload::NodeUp(_) => "NODE_UP",
            EventPayload::ShardAllocationChanged(_) => "SHARD_ALLOCATION_CHANGED",
            EventPayload::ShardAllocationNew(_) => "SHARD_ALLOCATION_NEW",
            EventPayload::SnapshotAgent(_) => "SNAPSHOT_AGENT",
            EventPayload::SnapshotAgentInfo(_) => "SNAPSHOT_AGENT_INFO",
            EventPayload::SnapshotDiscovery(_) => "SNAPSHOT_DISCOVERY",
            EventPayload::SnapshotNode(_) => "SNAPSHOT_NODE",
            EventPayload::SnapshotShard(_) => "SNAPSHOT_SHARD",
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use chrono::Utc;
    use serde_json;

    use super::super::ClusterDiscovery;
    use super::Event;

    #[test]
    fn from_json() {
        let payload = concat!(
            r#"{"event":"CLUSTER_NEW","data":{"cluster_id":"test","nodes":[]},"#,
            r#""timestamp":"2014-07-08T09:10:11.012Z"}"#
        );
        let event: Event = serde_json::from_str(&payload).unwrap();
        let discovery = ClusterDiscovery::new("test", vec![]);
        let expected = Event::builder()
            .timestamp(Utc.ymd(2014, 7, 8).and_hms_micro(9, 10, 11, 12000))
            .cluster()
            .cluster_new(discovery);
        assert_eq!(event, expected);
    }

    #[test]
    fn to_json() {
        let discovery = ClusterDiscovery::new("test", vec![]);
        let event = Event::builder()
            .timestamp(Utc.ymd(2014, 7, 8).and_hms(9, 10, 11))
            .cluster()
            .cluster_new(discovery);
        let payload = serde_json::to_string(&event).unwrap();
        let expected = concat!(
            r#"{"event":"CLUSTER_NEW","data":{"cluster_id":"test","display_name":null,"#,
            r#""nodes":[]},"timestamp":"2014-07-08T09:10:11Z"}"#
        );
        assert_eq!(payload, expected);
    }
}

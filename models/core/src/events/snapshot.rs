use serde_derive::Deserialize;
use serde_derive::Serialize;

use super::Event;
use super::EventBuilder;
use super::Payload;
use crate::agent::Agent;
use crate::agent::AgentInfo;
use crate::agent::Node;
use crate::agent::Shard;
use crate::cluster::discovery::ClusterDiscovery;

/// Enumerates all possible snapshot events emitted by the system.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
// TODO: use when possible #[non_exhaustive]
pub enum SnapshotEvent {
    /// Snapshot state of an agent status.
    #[serde(rename = "SNAPSHOT_AGENT")]
    Agent(Agent),

    /// Snapshot state of an agent info.
    #[serde(rename = "SNAPSHOT_AGENT_INFO")]
    AgentInfo(AgentInfo),

    /// Snapshot state of a cluster discovery.
    #[serde(rename = "SNAPSHOT_CLUSTER_DISCOVERY")]
    ClusterDiscovery(ClusterDiscovery),

    /// Snapshot state of a cluster node.
    #[serde(rename = "SNAPSHOT_NODE")]
    Node(Node),

    /// Snapshot state of a cluster shard.
    #[serde(rename = "SNAPSHOT_SHARD")]
    Shard(Shard),
}

impl SnapshotEvent {
    /// Look up the cluster ID for the event, if they have one.
    pub fn cluster_id(&self) -> Option<&str> {
        let cluster_id = match self {
            SnapshotEvent::Agent(agent) => &agent.cluster_id,
            SnapshotEvent::AgentInfo(info) => &info.cluster_id,
            SnapshotEvent::ClusterDiscovery(discovery) => &discovery.cluster_id,
            SnapshotEvent::Node(node) => &node.cluster_id,
            SnapshotEvent::Shard(shard) => &shard.cluster_id,
        };
        Some(cluster_id)
    }

    /// Returns the event "code", the string that represents the event type.
    pub fn code(&self) -> &'static str {
        match self {
            SnapshotEvent::Agent(_) => "SNAPSHOT_AGENT",
            SnapshotEvent::AgentInfo(_) => "SNAPSHOT_AGENT_INFO",
            SnapshotEvent::ClusterDiscovery(_) => "SNAPSHOT_CLUSTER_DISCOVERY",
            SnapshotEvent::Node(_) => "SNAPSHOT_NODE",
            SnapshotEvent::Shard(_) => "SNAPSHOT_SHARD",
        }
    }

    /// Returns the "ordering ID" for correctly streaming the event.
    pub fn stream_key(&self) -> &str {
        self.cluster_id().unwrap_or("<system>")
    }
}

/// Build `SnapshotEvent`s, validating inputs.
pub struct SnapshotEventBuilder {
    pub(super) builder: EventBuilder,
}

impl SnapshotEventBuilder {
    /// Build a `SnapshotEvent::Agent` event.
    pub fn agent(self, agent: Agent) -> Event {
        let event = SnapshotEvent::Agent(agent);
        let payload = Payload::Snapshot(event);
        self.builder.finish(payload)
    }

    /// Build a `SnapshotEvent::AgentInfo` event.
    pub fn agent_info(self, agent: AgentInfo) -> Event {
        let event = SnapshotEvent::AgentInfo(agent);
        let payload = Payload::Snapshot(event);
        self.builder.finish(payload)
    }

    /// Build a `SnapshotEvent::ClusterDiscovery` event.
    pub fn discovery(self, cluster: ClusterDiscovery) -> Event {
        let event = SnapshotEvent::ClusterDiscovery(cluster);
        let payload = Payload::Snapshot(event);
        self.builder.finish(payload)
    }

    /// Build a `SnapshotEvent::Node` event.
    pub fn node(self, node: Node) -> Event {
        let event = SnapshotEvent::Node(node);
        let payload = Payload::Snapshot(event);
        self.builder.finish(payload)
    }

    /// Build a `SnapshotEvent::Shard` event.
    pub fn shard(self, shard: Shard) -> Event {
        let event = SnapshotEvent::Shard(shard);
        let payload = Payload::Snapshot(event);
        self.builder.finish(payload)
    }
}

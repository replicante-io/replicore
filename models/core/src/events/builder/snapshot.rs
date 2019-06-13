use super::super::super::Agent;
use super::super::super::AgentInfo;
use super::super::super::ClusterDiscovery;
use super::super::super::Node;
use super::super::super::Shard;

use super::super::Event;
use super::super::EventBuilder;
use super::super::EventPayload;

/// Build `Event`s that belongs to the snapshot family.
pub struct SnapshotBuilder {
    builder: EventBuilder,
}

impl SnapshotBuilder {
    /// Create a new snapshot event builder.
    pub fn builder(builder: EventBuilder) -> SnapshotBuilder {
        SnapshotBuilder { builder }
    }

    /// Build an `EventPayload::SnapshotAgent` event.
    pub fn agent(self, agent: Agent) -> Event {
        let data = EventPayload::SnapshotAgent(agent);
        self.builder.build(data)
    }

    /// Build an `EventPayload::SnapshotAgentInfo` event.
    pub fn agent_info(self, agent: AgentInfo) -> Event {
        let data = EventPayload::SnapshotAgentInfo(agent);
        self.builder.build(data)
    }

    /// Build an `EventPayload::SnapshotDiscovery` event.
    pub fn discovery(self, cluster: ClusterDiscovery) -> Event {
        let data = EventPayload::SnapshotDiscovery(cluster);
        self.builder.build(data)
    }

    /// Build an `EventPayload::SnapshotNode` event.
    pub fn node(self, node: Node) -> Event {
        let data = EventPayload::SnapshotNode(node);
        self.builder.build(data)
    }

    /// Build an `EventPayload::SnapshotShard` event.
    pub fn shard(self, shard: Shard) -> Event {
        let data = EventPayload::SnapshotShard(shard);
        self.builder.build(data)
    }
}

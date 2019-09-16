use crate::agent::Agent;
use crate::agent::AgentInfo;
use crate::agent::Node;
use crate::agent::Shard;
use crate::cluster::ClusterDiscovery;

use crate::events::Event;
use crate::events::EventBuilder;
use crate::events::EventPayload;

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

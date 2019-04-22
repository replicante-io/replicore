use bson::UtcDateTime;

use replicante_data_models::AgentInfo;
use replicante_data_models::Event;
use replicante_data_models::EventPayload;
use replicante_data_models::Node;
use replicante_data_models::Shard;

/// Wrap an `AgentInfo` with store only and MongoDB specific fields.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct AgentInfoDocument {
    #[serde(flatten)]
    pub agent: AgentInfo,

    /// If `true` the model was NOT updated by the last cluster state refresh operation.
    pub stale: bool,
}

impl From<AgentInfo> for AgentInfoDocument {
    fn from(agent: AgentInfo) -> AgentInfoDocument {
        AgentInfoDocument {
            agent,
            stale: false,
        }
    }
}

impl From<AgentInfoDocument> for AgentInfo {
    fn from(wrapper: AgentInfoDocument) -> AgentInfo {
        wrapper.agent
    }
}

/// Wrap an `Event` to allow BSON to encode/decode timestamps correctly.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct EventDocument {
    #[serde(flatten)]
    pub payload: EventPayload,
    pub timestamp: UtcDateTime,
}

impl From<Event> for EventDocument {
    fn from(event: Event) -> EventDocument {
        EventDocument {
            payload: event.payload,
            timestamp: UtcDateTime(event.timestamp),
        }
    }
}

impl From<EventDocument> for Event {
    fn from(event: EventDocument) -> Event {
        Event {
            payload: event.payload,
            timestamp: event.timestamp.0,
        }
    }
}

/// Wraps a `Node` with store only and MongoDB specific fields.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct NodeDocument {
    #[serde(flatten)]
    pub node: Node,

    /// If `true` the model was NOT updated by the last cluster state refresh operation.
    pub stale: bool,
}

impl From<Node> for NodeDocument {
    fn from(node: Node) -> NodeDocument {
        NodeDocument {
            node,
            stale: false,
        }
    }
}

impl From<NodeDocument> for Node {
    fn from(wrapper: NodeDocument) -> Node {
        wrapper.node
    }
}

/// Wraps a `Shard` with store only and MongoDB specific fields.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ShardDocument {
    #[serde(flatten)]
    pub shard: Shard,

    /// If `true` the model was NOT updated by the last cluster state refresh operation.
    pub stale: bool,
}

impl From<Shard> for ShardDocument {
    fn from(shard: Shard) -> ShardDocument {
        ShardDocument {
            shard,
            stale: false,
        }
    }
}

impl From<ShardDocument> for Shard {
    fn from(wrapper: ShardDocument) -> Shard {
        wrapper.shard
    }
}

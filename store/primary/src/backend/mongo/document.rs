use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_models_core::AgentInfo;
use replicante_models_core::Node;
use replicante_models_core::Shard;

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
        NodeDocument { node, stale: false }
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

use std::collections::HashMap;

use bson::UtcDateTime;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_models_core::actions::Action;
use replicante_models_core::actions::ActionRequester;
use replicante_models_core::actions::ActionState;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::Node;
use replicante_models_core::agent::Shard;

/// Wrap an `Action` with store only fields and MongoDB specific types.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionDocument {
    // ID attributes.
    pub cluster_id: String,
    pub node_id: String,
    pub action_id: String,

    // Record attributes.
    pub created_ts: UtcDateTime,
    pub finished_ts: Option<UtcDateTime>,
    pub headers: HashMap<String, String>,
    pub kind: String,
    pub refresh_id: i64,
    pub requester: ActionRequester,
    pub schedule_attempt: i32,
    pub scheduled_ts: Option<UtcDateTime>,
    pub state: ActionState,

    // The encoded JSON form uses unsigned integers which are not supported by BSON.
    // For this reason store JSON as a String and transparently encode/decode.
    pub args: String,
    pub state_payload: Option<String>,
}

impl From<Action> for ActionDocument {
    fn from(action: Action) -> ActionDocument {
        let args =
            serde_json::to_string(&action.args).expect("serde_json::Value not converted to String");
        let state_payload = action.state_payload.map(|payload| {
            serde_json::to_string(&payload).expect("serde_json::Value not converted to String")
        });
        ActionDocument {
            action_id: action.action_id.to_string(),
            args,
            cluster_id: action.cluster_id,
            created_ts: UtcDateTime(action.created_ts),
            finished_ts: action.finished_ts.map(UtcDateTime),
            headers: action.headers,
            kind: action.kind,
            node_id: action.node_id,
            refresh_id: action.refresh_id,
            requester: action.requester,
            schedule_attempt: action.schedule_attempt,
            scheduled_ts: action.scheduled_ts.map(UtcDateTime),
            state: action.state,
            state_payload,
        }
    }
}

impl From<ActionDocument> for Action {
    fn from(action: ActionDocument) -> Action {
        let action_id = action
            .action_id
            .parse()
            .expect("Action ID not converted to UUID");
        let args =
            serde_json::from_str(&action.args).expect("String not converted to serde_json::Value");
        let state_payload = action.state_payload.map(|payload| {
            serde_json::from_str(&payload).expect("String not converted to serde_json::Value")
        });
        Action {
            action_id,
            args,
            cluster_id: action.cluster_id,
            created_ts: action.created_ts.0,
            finished_ts: action.finished_ts.map(|ts| ts.0),
            headers: action.headers,
            kind: action.kind,
            node_id: action.node_id,
            refresh_id: action.refresh_id,
            requester: action.requester,
            schedule_attempt: action.schedule_attempt,
            scheduled_ts: action.scheduled_ts.map(|ts| ts.0),
            state: action.state,
            state_payload,
        }
    }
}

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

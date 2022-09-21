use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::actions::node::Action;
use crate::actions::node::ActionState;

/// Summary information about `NodeAction`s.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct NodeActionSummary {
    // ID attributes.
    pub cluster_id: String,
    pub node_id: String,
    pub action_id: Uuid,

    // Summary attributes.
    /// Timestamp of action creation.
    pub created_ts: DateTime<Utc>,
    /// Timestamp action entered a final state (success or failure).
    pub finished_ts: Option<DateTime<Utc>>,
    /// Identifier of the node action logic to execute.
    pub kind: String,
    /// Timestamp the action was scheduled on the agent.
    pub scheduled_ts: Option<DateTime<Utc>>,
    /// State the action is currently in.
    pub state: ActionState,
}

impl From<Action> for NodeActionSummary {
    fn from(action: Action) -> NodeActionSummary {
        NodeActionSummary {
            cluster_id: action.cluster_id,
            node_id: action.node_id,
            action_id: action.action_id,
            created_ts: action.created_ts,
            finished_ts: action.finished_ts,
            kind: action.kind,
            scheduled_ts: action.scheduled_ts,
            state: action.state,
        }
    }
}

/// API Response for NodeAction summary listing.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct NodeActionSummariesResponse {
    pub actions: Vec<NodeActionSummary>,
}

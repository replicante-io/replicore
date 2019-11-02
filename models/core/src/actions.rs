use std::collections::HashMap;

use chrono::DateTime;
use chrono::Utc;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value as Json;
use uuid::Uuid;

use replicante_models_agent::actions::ActionHistoryItem;
use replicante_models_agent::actions::ActionModel as ActionWire;
use replicante_models_agent::actions::ActionState as ActionStateWire;

pub use replicante_models_agent::actions::ActionRequester;

/// Action state and metadata information.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Action {
    // ID attributes.
    pub cluster_id: String,
    pub node_id: String,
    pub action_id: Uuid,

    // Record attributes.
    pub args: Json,
    pub created_ts: DateTime<Utc>,
    pub finished_ts: Option<DateTime<Utc>>,
    pub headers: HashMap<String, String>,
    pub kind: String,
    pub refresh_id: i64,
    pub requester: ActionRequester,
    pub state: ActionState,
    pub state_payload: Option<Json>,
}

impl Action {
    pub fn new<S1, S2>(cluster_id: S1, node_id: S2, refresh_id: i64, action: ActionWire) -> Action
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Action {
            action_id: action.id,
            args: action.args,
            cluster_id: cluster_id.into(),
            created_ts: action.created_ts,
            finished_ts: action.finished_ts,
            headers: action.headers,
            kind: action.kind,
            node_id: node_id.into(),
            refresh_id,
            requester: action.requester,
            state: action.state.into(),
            state_payload: action.state_payload,
        }
    }
}

/// Action history metadata and transitions.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionHistory {
    // ID attributes.
    pub cluster_id: String,
    pub node_id: String,
    pub action_id: Uuid,

    // Action history attributes.
    pub finished_ts: Option<DateTime<Utc>>,
    pub timestamp: DateTime<Utc>,
    pub state: ActionState,
    pub state_payload: Option<Json>,
}

impl ActionHistory {
    pub fn new<S1, S2>(cluster_id: S1, node_id: S2, history: ActionHistoryItem) -> ActionHistory
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ActionHistory {
            cluster_id: cluster_id.into(),
            node_id: node_id.into(),
            action_id: history.action_id,
            finished_ts: None,
            timestamp: history.timestamp,
            state: history.state.into(),
            state_payload: history.state_payload,
        }
    }
}

/// Current state of an action execution.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum ActionState {
    /// The action is to be cancelled, but that has not happened yet.
    #[serde(rename = "CANCEL")]
    Cancel,

    /// The action was successfully cancelled.
    #[serde(rename = "CANCELLED")]
    Cancelled,

    /// The action was successfully completed.
    #[serde(rename = "DONE")]
    Done,

    /// The action ended with an error.
    #[serde(rename = "FAILED")]
    Failed,

    /// The non-finished action was no longer reported by the agent.
    #[serde(rename = "LOST")]
    Lost,

    /// The action has just been sheduled and is not being executed yet.
    #[serde(rename = "NEW")]
    New,

    /// The action was started by the agent and is in progress.
    #[serde(rename = "RUNNING")]
    Running,
}

impl From<ActionStateWire> for ActionState {
    fn from(state: ActionStateWire) -> ActionState {
        match state {
            ActionStateWire::Cancel => ActionState::Cancel,
            ActionStateWire::Cancelled => ActionState::Cancelled,
            ActionStateWire::Done => ActionState::Done,
            ActionStateWire::Failed => ActionState::Failed,
            ActionStateWire::New => ActionState::New,
            ActionStateWire::Running => ActionState::Running,
        }
    }
}

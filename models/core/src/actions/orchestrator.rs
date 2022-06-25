use std::collections::HashMap;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as Json;
use uuid::Uuid;

/// Orchestrator action state and metadata information.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct OrchestratorAction {
    // ID attributes.
    pub cluster_id: String,
    pub action_id: Uuid,

    // Record attributes.
    /// Action-dependent arguments to execute the action with.
    pub args: Json,
    /// Timestamp of action creation.
    pub created_ts: DateTime<Utc>,
    /// Timestamp action entered a final state (success or failure).
    pub finished_ts: Option<DateTime<Utc>>,
    /// Arbitrary key/value headers attached to the action.
    pub headers: HashMap<String, String>,
    /// Identifier of the orchestrator action logic to execute.
    pub kind: String,
    /// Timestamp the action was scheduled (stared) in Replicante Core.
    pub scheduled_ts: Option<DateTime<Utc>>,
    /// State the action is currently in.
    pub state: OrchestratorActionState,
    /// Action-dependent state data, if the action needs to persist state.
    pub state_payload: Option<Json>,
}

/// Metadata to describe orchestrator actions (mainly to humans).
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct OrchestratorActionDescriptor {
    pub summary: String,
}

/// Current state of an orchestrator action execution.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum OrchestratorActionState {
    /// The action was interrupted or never executed.
    #[serde(rename = "CANCELLED")]
    Cancelled,

    /// The action finished successfully.
    #[serde(rename = "DONE")]
    Done,

    /// Unable to successfully execute the action.
    #[serde(rename = "FAILED")]
    Failed,

    /// Replicante is waiting for a user to approve the action before scheduling it.
    #[serde(rename = "PENDING_APPROVE")]
    PendingApprove,

    /// Replicante knows about the action and may or may not have sent it to the Agent.
    #[serde(rename = "PENDING_SCHEDULE")]
    PendingSchedule,

    /// The action is running on the Replicante Agent.
    #[serde(rename = "RUNNING")]
    Running,
}

impl OrchestratorActionState {
    /// Check if the action is running or sent to the agent to run.
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }
}

impl std::fmt::Display for OrchestratorActionState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Cancelled => write!(f, "CANCELLED"),
            Self::Done => write!(f, "DONE"),
            Self::Failed => write!(f, "FAILED"),
            Self::PendingApprove => write!(f, "PENDING_APPROVE"),
            Self::PendingSchedule => write!(f, "PENDING_SCHEDULE"),
            Self::Running => write!(f, "RUNNING"),
        }
    }
}

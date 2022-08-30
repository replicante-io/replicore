use std::collections::HashMap;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as Json;
use uuid::Uuid;

/// Orchestrator action state and metadata information.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
    /// Information about errors encountered when executing the action.
    pub state_payload_error: Option<Json>,
    /// Timeout after which the running action is failed. Overrides the handler's default.
    #[serde(default)]
    pub timeout: Option<Duration>,
}

impl OrchestratorAction {
    /// Mark the action as finished and sets the finish timestamp to now.
    pub fn finish(&mut self, state: OrchestratorActionState) {
        self.state = state;
        self.finished_ts = Some(Utc::now());
    }
}

/// Metadata attached to and orchestrator action (for both humans and the system).
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct OrchestratorActionMetadata {
    /// Concurrent scheduling mode supported by the action.
    pub schedule_mode: OrchestratorActionScheduleMode,

    /// A short summary of what the action does, for human consumption.
    pub summary: String,

    /// Default timeout after which running orchestrator actions are failed.
    pub timeout: Duration,
}

/// Possible scheduling modes for orchestrator actions.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum OrchestratorActionScheduleMode {
    /// The action must be the only thing running in the cluster.
    ///
    /// Scheduling of the action will wait for running (orchestrator and node) actions to complete.
    /// Scheduling of other (orchestrator and node) actions is blocked until the action is complete.
    #[serde(rename = "EXCLUSIVE")]
    Exclusive,
}

impl OrchestratorActionScheduleMode {
    /// Indicate this action mode is exclusive across all scheduling modes.
    ///
    /// Exclusive actions expect to be the only ones running regardless of scheduling mode
    /// of other actions.
    /// When an exclusive action is started no further action should be started.
    pub fn is_exclusive(&self) -> bool {
        match self {
            Self::Exclusive => true,
        }
    }

    /// Indicate this action mode is exclusive across actions with the same mode.
    ///
    /// Exclusive with mode actions expect to be the only ones running for their schedule mode.
    /// When an exclusive with mode action is started no further action with the same mode
    /// should be started but actions with other mode can start.
    pub fn is_exclusive_with_mode(&self) -> bool {
        match self {
            Self::Exclusive => true,
        }
    }
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
    /// Check if the action is in a final state (done, failed, ...).
    pub fn is_final(&self) -> bool {
        matches!(self, Self::Cancelled | Self::Done | Self::Failed)
    }

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

/// Sync-needed information about an orchestrator action.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OrchestratorActionSyncSummary {
    pub cluster_id: String,
    pub action_id: Uuid,
    pub kind: String,
    pub state: OrchestratorActionState,
}

impl From<&OrchestratorAction> for OrchestratorActionSyncSummary {
    fn from(action: &OrchestratorAction) -> OrchestratorActionSyncSummary {
        OrchestratorActionSyncSummary {
            cluster_id: action.cluster_id.clone(),
            action_id: action.action_id,
            kind: action.kind.clone(),
            state: action.state,
        }
    }
}

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::actions::orchestrator::OrchestratorAction;
use crate::actions::orchestrator::OrchestratorActionState;

/// Summary information about `OrchestratorAction`s.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct OrchestratorActionSummary {
    // ID attributes.
    pub cluster_id: String,
    pub action_id: Uuid,

    // Summary attributes.
    /// Timestamp of action creation.
    pub created_ts: DateTime<Utc>,
    /// Timestamp action entered a final state (success or failure).
    pub finished_ts: Option<DateTime<Utc>>,
    /// Identifier of the orchestrator action logic to execute.
    pub kind: String,
    /// State the action is currently in.
    pub state: OrchestratorActionState,
}

impl From<OrchestratorAction> for OrchestratorActionSummary {
    fn from(action: OrchestratorAction) -> OrchestratorActionSummary {
        OrchestratorActionSummary {
            cluster_id: action.cluster_id,
            action_id: action.action_id,
            created_ts: action.created_ts,
            finished_ts: action.finished_ts,
            kind: action.kind,
            state: action.state,
        }
    }
}

/// API Response for OrchestratorAction summary listing.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct OrchestratorActionSummariesResponse {
    pub actions: Vec<OrchestratorActionSummary>,
}

use std::collections::HashMap;
use std::time::Duration;

use bson::DateTime;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_models_core::actions::node::Action;
use replicante_models_core::actions::node::ActionRequester;
use replicante_models_core::actions::node::ActionState;
use replicante_models_core::actions::orchestrator::OrchestratorAction;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicante_models_core::api::orchestrator_action::OrchestratorActionSummary;
use replicante_models_core::cluster::discovery::DiscoverySettings;
use replicante_models_core::cluster::ClusterSettings;

/// Wrap an `Action` with store only fields and MongoDB specific types.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActionDocument {
    // ID attributes.
    pub cluster_id: String,
    pub node_id: String,
    pub action_id: String,

    // Record attributes.
    pub created_ts: DateTime,
    pub finished_ts: Option<DateTime>,
    pub headers: HashMap<String, String>,
    pub kind: String,
    pub requester: ActionRequester,
    pub schedule_attempt: i32,
    pub scheduled_ts: Option<DateTime>,
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
            created_ts: DateTime::from(action.created_ts),
            finished_ts: action.finished_ts.map(DateTime::from),
            headers: action.headers,
            kind: action.kind,
            node_id: action.node_id,
            requester: action.requester,
            schedule_attempt: action.schedule_attempt,
            scheduled_ts: action.scheduled_ts.map(DateTime::from),
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
            requester: action.requester,
            schedule_attempt: action.schedule_attempt,
            scheduled_ts: action.scheduled_ts.map(|ts| ts.0),
            state: action.state,
            state_payload,
        }
    }
}

/// Wraps a `ClusterSettings` with store only fields.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ClusterSettingsDocument {
    #[serde(flatten)]
    pub settings: ClusterSettings,

    /// Timestamp for the next expected discovery run.
    pub next_orchestrate: Option<DateTime>,
}

impl From<ClusterSettings> for ClusterSettingsDocument {
    fn from(settings: ClusterSettings) -> ClusterSettingsDocument {
        ClusterSettingsDocument {
            settings,
            next_orchestrate: None,
        }
    }
}

impl From<ClusterSettingsDocument> for ClusterSettings {
    fn from(document: ClusterSettingsDocument) -> ClusterSettings {
        document.settings
    }
}

/// Wraps a `DiscoverySettings` with store only fields.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct DiscoverySettingsDocument {
    #[serde(flatten)]
    pub settings: DiscoverySettings,

    /// Timestamp for the next expected discovery run.
    pub next_run: Option<DateTime>,
}

impl From<DiscoverySettings> for DiscoverySettingsDocument {
    fn from(settings: DiscoverySettings) -> DiscoverySettingsDocument {
        DiscoverySettingsDocument {
            settings,
            next_run: None,
        }
    }
}

impl From<DiscoverySettingsDocument> for DiscoverySettings {
    fn from(document: DiscoverySettingsDocument) -> DiscoverySettings {
        document.settings
    }
}

/// Wrap an `OrchestratorAction` with store only fields and MongoDB specific types.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OrchestratorActionDocument {
    // ID attributes.
    pub cluster_id: String,
    pub action_id: String,

    // Record attributes.
    pub created_ts: DateTime,
    pub finished_ts: Option<DateTime>,
    pub headers: HashMap<String, String>,
    pub kind: String,
    pub scheduled_ts: Option<DateTime>,
    pub state: OrchestratorActionState,
    pub timeout: Option<Duration>,

    // The encoded JSON form uses unsigned integers which are not supported by BSON.
    // For this reason store JSON as a String and transparently encode/decode.
    pub args: String,
    pub state_payload: Option<String>,
    pub state_payload_error: Option<String>,
}

impl From<OrchestratorAction> for OrchestratorActionDocument {
    fn from(action: OrchestratorAction) -> OrchestratorActionDocument {
        let args =
            serde_json::to_string(&action.args).expect("serde_json::Value not converted to String");
        let state_payload = action.state_payload.map(|payload| {
            serde_json::to_string(&payload).expect("serde_json::Value not converted to String")
        });
        let state_payload_error = action.state_payload_error.map(|payload| {
            serde_json::to_string(&payload).expect("serde_json::Value not converted to String")
        });
        OrchestratorActionDocument {
            action_id: action.action_id.to_string(),
            args,
            cluster_id: action.cluster_id,
            created_ts: DateTime::from(action.created_ts),
            finished_ts: action.finished_ts.map(DateTime::from),
            headers: action.headers,
            kind: action.kind,
            scheduled_ts: action.scheduled_ts.map(DateTime::from),
            state: action.state,
            state_payload,
            state_payload_error,
            timeout: action.timeout,
        }
    }
}

impl From<OrchestratorActionDocument> for OrchestratorAction {
    fn from(action: OrchestratorActionDocument) -> OrchestratorAction {
        let action_id = action
            .action_id
            .parse()
            .expect("Action ID not converted to UUID");
        let args =
            serde_json::from_str(&action.args).expect("String not converted to serde_json::Value");
        let state_payload = action.state_payload.map(|payload| {
            serde_json::from_str(&payload).expect("String not converted to serde_json::Value")
        });
        let state_payload_error = action.state_payload_error.map(|payload| {
            serde_json::from_str(&payload).expect("String not converted to serde_json::Value")
        });
        OrchestratorAction {
            action_id,
            args,
            cluster_id: action.cluster_id,
            created_ts: action.created_ts.0,
            finished_ts: action.finished_ts.map(|ts| ts.0),
            headers: action.headers,
            kind: action.kind,
            scheduled_ts: action.scheduled_ts.map(|ts| ts.0),
            state: action.state,
            state_payload,
            state_payload_error,
            timeout: action.timeout,
        }
    }
}

/// Wrap an `OrchestratorActionSummary` with store only fields and MongoDB specific types.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OrchestratorActionSummaryDocument {
    pub cluster_id: String,
    pub action_id: String,
    pub created_ts: DateTime,
    pub finished_ts: Option<DateTime>,
    pub kind: String,
    pub state: OrchestratorActionState,
}

impl From<OrchestratorActionSummaryDocument> for OrchestratorActionSummary {
    fn from(action: OrchestratorActionSummaryDocument) -> OrchestratorActionSummary {
        let action_id = action
            .action_id
            .parse()
            .expect("Action ID not converted to UUID");
        OrchestratorActionSummary {
            action_id,
            cluster_id: action.cluster_id,
            created_ts: action.created_ts.0,
            finished_ts: action.finished_ts.map(|ts| ts.0),
            kind: action.kind,
            state: action.state,
        }
    }
}

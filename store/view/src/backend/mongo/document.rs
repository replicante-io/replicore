use std::collections::HashMap;
use std::time::Duration;

use mongodb::bson::DateTime;
use serde::Deserialize;
use serde::Serialize;

use replicante_models_core::actions::node::Action;
use replicante_models_core::actions::node::ActionHistory;
use replicante_models_core::actions::node::ActionHistoryOrigin;
use replicante_models_core::actions::node::ActionRequester;
use replicante_models_core::actions::node::ActionState;
use replicante_models_core::actions::orchestrator::OrchestratorAction;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicante_models_core::cluster::OrchestrateReport;
use replicante_models_core::cluster::OrchestrateReportOutcome;
use replicante_models_core::cluster::SchedChoice;
use replicante_models_core::events::Event;
use replicante_models_core::events::Payload;

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
            created_ts: action.created_ts.to_chrono(),
            finished_ts: action.finished_ts.map(DateTime::to_chrono),
            headers: action.headers,
            kind: action.kind,
            node_id: action.node_id,
            requester: action.requester,
            schedule_attempt: action.schedule_attempt,
            scheduled_ts: action.scheduled_ts.map(DateTime::to_chrono),
            state: action.state,
            state_payload,
        }
    }
}

/// Wrap an `ActionHistory` with MongoDB specific types.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActionHistoryDocument {
    // ID attributes.
    pub cluster_id: String,
    pub node_id: String,
    pub action_id: String,

    // Action history attributes.
    pub finished_ts: Option<DateTime>,
    pub origin: ActionHistoryOrigin,
    pub timestamp: DateTime,
    pub state: ActionState,

    // The encoded JSON form uses unsigned integers which are not supported by BSON.
    // For this reason store JSON as a String and transparently encode/decode.
    pub state_payload: Option<String>,
}

impl From<ActionHistory> for ActionHistoryDocument {
    fn from(history: ActionHistory) -> ActionHistoryDocument {
        let state_payload = history.state_payload.map(|payload| {
            serde_json::to_string(&payload).expect("serde_json::Value not converted to String")
        });
        ActionHistoryDocument {
            cluster_id: history.cluster_id,
            node_id: history.node_id,
            action_id: history.action_id.to_string(),
            origin: history.origin,
            finished_ts: history.finished_ts.map(DateTime::from),
            timestamp: DateTime::from(history.timestamp),
            state: history.state,
            state_payload,
        }
    }
}

impl From<ActionHistoryDocument> for ActionHistory {
    fn from(history: ActionHistoryDocument) -> ActionHistory {
        let action_id = history
            .action_id
            .parse()
            .expect("Action ID not converted to UUID");
        let state_payload = history.state_payload.map(|payload| {
            serde_json::from_str(&payload).expect("String not converted to serde_json::Value")
        });
        ActionHistory {
            cluster_id: history.cluster_id,
            node_id: history.node_id,
            action_id,
            finished_ts: history.finished_ts.map(DateTime::to_chrono),
            origin: history.origin,
            timestamp: history.timestamp.to_chrono(),
            state: history.state,
            state_payload,
        }
    }
}

/// Wrap an `Event` to allow BSON to encode/decode timestamps correctly.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EventDocument {
    #[serde(flatten)]
    pub payload: Payload,
    pub timestamp: DateTime,
}

impl From<Event> for EventDocument {
    fn from(event: Event) -> EventDocument {
        EventDocument {
            payload: event.payload,
            timestamp: DateTime::from(event.timestamp),
        }
    }
}

impl From<EventDocument> for Event {
    fn from(event: EventDocument) -> Event {
        Event {
            payload: event.payload,
            timestamp: event.timestamp.to_chrono(),
        }
    }
}

/// Wrap an `OrchestrateReport` to allow BSON to encode/decode timestamps correctly.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OrchestrateReportDocument {
    // Cluster identification attributes.
    pub namespace: String,
    pub cluster_id: String,

    // Orchestration task metadata.
    pub start_time: DateTime,
    pub duration: Duration,
    pub outcome: OrchestrateReportOutcome,

    // Orchestration task details.
    pub action_scheduling_choices: Option<SchedChoice>,
    pub node_actions_lost: u64,
    pub node_actions_schedule_failed: u64,
    pub node_actions_scheduled: u64,
    pub nodes_failed: u64,
    pub nodes_synced: u64,
}

impl From<OrchestrateReport> for OrchestrateReportDocument {
    fn from(report: OrchestrateReport) -> OrchestrateReportDocument {
        OrchestrateReportDocument {
            namespace: report.namespace,
            cluster_id: report.cluster_id,
            start_time: DateTime::from(report.start_time),
            duration: report.duration,
            outcome: report.outcome,
            action_scheduling_choices: report.action_scheduling_choices,
            node_actions_lost: report.node_actions_lost,
            node_actions_schedule_failed: report.node_actions_schedule_failed,
            node_actions_scheduled: report.node_actions_scheduled,
            nodes_failed: report.nodes_failed,
            nodes_synced: report.nodes_synced,
        }
    }
}

impl From<OrchestrateReportDocument> for OrchestrateReport {
    fn from(document: OrchestrateReportDocument) -> OrchestrateReport {
        OrchestrateReport {
            namespace: document.namespace,
            cluster_id: document.cluster_id,
            start_time: document.start_time.to_chrono(),
            duration: document.duration,
            outcome: document.outcome,
            action_scheduling_choices: document.action_scheduling_choices,
            node_actions_lost: document.node_actions_lost,
            node_actions_schedule_failed: document.node_actions_schedule_failed,
            node_actions_scheduled: document.node_actions_scheduled,
            nodes_failed: document.nodes_failed,
            nodes_synced: document.nodes_synced,
        }
    }
}

/// Wrap an `Action` with store only fields and MongoDB specific types.
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
            created_ts: action.created_ts.to_chrono(),
            finished_ts: action.finished_ts.map(DateTime::to_chrono),
            headers: action.headers,
            kind: action.kind,
            scheduled_ts: action.scheduled_ts.map(DateTime::to_chrono),
            state: action.state,
            state_payload,
            state_payload_error,
            timeout: action.timeout,
        }
    }
}

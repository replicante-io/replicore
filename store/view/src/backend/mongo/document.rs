use std::collections::HashMap;

use bson::UtcDateTime;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_models_core::actions::Action;
use replicante_models_core::actions::ActionHistory;
use replicante_models_core::actions::ActionHistoryOrigin;
use replicante_models_core::actions::ActionRequester;
use replicante_models_core::actions::ActionState;
use replicante_models_core::events::Event;
use replicante_models_core::events::Payload;

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
            scheduled_ts: action.scheduled_ts.map(|ts| ts.0),
            state: action.state,
            state_payload,
        }
    }
}

/// Wrap an `ActionHistory` with MongoDB specific types.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionHistoryDocument {
    // ID attributes.
    pub cluster_id: String,
    pub node_id: String,
    pub action_id: String,

    // Action history attributes.
    pub finished_ts: Option<UtcDateTime>,
    pub origin: ActionHistoryOrigin,
    pub timestamp: UtcDateTime,
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
            finished_ts: history.finished_ts.map(UtcDateTime),
            timestamp: UtcDateTime(history.timestamp),
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
            finished_ts: history.finished_ts.map(|ts| ts.0),
            origin: history.origin,
            timestamp: history.timestamp.0,
            state: history.state,
            state_payload,
        }
    }
}

/// Wrap an `Event` to allow BSON to encode/decode timestamps correctly.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EventDocument {
    #[serde(flatten)]
    pub payload: Payload,
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

use chrono::DateTime;
use chrono::Utc;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use uuid::Uuid;

use replicante_models_agent::actions::ActionHistoryItem;

use super::Event;
use super::EventBuilder;
use super::Payload;
use crate::actions::Action;

/// Hold data about an action change with before and after state.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionChanged {
    pub cluster_id: String,
    pub current: Action,
    pub previous: Action,
}

/// Hold data about an action history from the agent.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionHistory {
    pub action_id: Uuid,
    pub cluster_id: String,
    pub finished_ts: Option<DateTime<Utc>>,
    pub history: Vec<ActionHistoryItem>,
    pub node_id: String,
}

/// Enumerates all possible action events emitted by the system.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
#[allow(clippy::large_enum_variant)]
// TODO: use when possible #[non_exhaustive]
pub enum ActionEvent {
    /// An action change was observed.
    #[serde(rename = "ACTION_CHANGED")]
    Changed(Box<ActionChanged>),

    /// An action has finished, the latest action state is attached.
    #[serde(rename = "ACTION_FINISHED")]
    Finished(Action),

    /// Snapshot of an action's history.
    #[serde(rename = "ACTION_HISTORY")]
    History(ActionHistory),

    /// An unfinished action was no longer reported by the originating agent.
    #[serde(rename = "ACTION_LOST")]
    Lost(Action),

    /// An action was discovered for the first time.
    #[serde(rename = "ACTION_NEW")]
    New(Action),
}

impl ActionEvent {
    /// Look up the cluster ID for the event, if they have one.
    pub fn cluster_id(&self) -> Option<&str> {
        let cluster_id = match self {
            ActionEvent::Changed(change) => &change.cluster_id,
            ActionEvent::Finished(action) => &action.cluster_id,
            ActionEvent::History(info) => &info.cluster_id,
            ActionEvent::Lost(action) => &action.cluster_id,
            ActionEvent::New(action) => &action.cluster_id,
        };
        Some(cluster_id)
    }

    /// Returns the event "code", the string that represents the event type.
    pub fn code(&self) -> &'static str {
        match self {
            ActionEvent::Changed(_) => "ACTION_CHANGED",
            ActionEvent::Finished(_) => "ACTION_FINISHED",
            ActionEvent::History(_) => "ACTION_HISTORY",
            ActionEvent::Lost(_) => "ACTION_LOST",
            ActionEvent::New(_) => "ACTION_NEW",
        }
    }

    /// Returns the "ordering ID" for correctly streaming the event.
    pub fn stream_key(&self) -> &str {
        self.cluster_id().unwrap_or("<system>")
    }
}

/// Build `ActionEvent`s, validating inputs.
pub struct ActionEventBuilder {
    pub(super) builder: EventBuilder,
}

impl ActionEventBuilder {
    /// Build an `ActionEvent::Changed` event.
    pub fn changed(self, previous: Action, current: Action) -> Event {
        let event = ActionEvent::Changed(Box::new(ActionChanged {
            cluster_id: previous.cluster_id.clone(),
            current,
            previous,
        }));
        let payload = Payload::Action(event);
        self.builder.finish(payload)
    }

    /// Build an `ActionEvent::Finished` event.
    pub fn finished(self, action: Action) -> Event {
        let event = ActionEvent::Finished(action);
        let payload = Payload::Action(event);
        self.builder.finish(payload)
    }

    /// Build an `ActionEvent::History` event.
    pub fn history(
        self,
        cluster_id: String,
        node_id: String,
        action_id: Uuid,
        finished_ts: Option<DateTime<Utc>>,
        history: Vec<ActionHistoryItem>,
    ) -> Event {
        let info = ActionHistory {
            action_id,
            cluster_id,
            finished_ts,
            history,
            node_id,
        };
        let event = ActionEvent::History(info);
        let payload = Payload::Action(event);
        self.builder.finish(payload)
    }

    /// Build an `ActionEvent::Lost` event.
    pub fn lost(self, action: Action) -> Event {
        let event = ActionEvent::Lost(action);
        let payload = Payload::Action(event);
        self.builder.finish(payload)
    }

    /// Build an `ActionEvent::New` event.
    pub fn new_action(self, action: Action) -> Event {
        let event = ActionEvent::New(action);
        let payload = Payload::Action(event);
        self.builder.finish(payload)
    }
}

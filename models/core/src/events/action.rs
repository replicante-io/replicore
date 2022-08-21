use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use replicante_models_agent::actions::ActionHistoryItem;

use super::Event;
use super::EventBuilder;
use super::Payload;
use crate::actions::node::Action;
use crate::actions::orchestrator::OrchestratorAction;
use crate::scope::EntityId;
use crate::scope::Namespace;

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

    /// An orchestrator action was changed.
    #[serde(rename = "ACTION_ORCHESTRATOR_CHANGED")]
    OrchestratorChanged(Box<OrchestratorActionChanged>),

    /// An orchestrator action has finished.
    #[serde(rename = "ACTION_ORCHESTRATOR_FINISHED")]
    OrchestratorFinished(OrchestratorAction),

    /// An orchestrator action was created.
    #[serde(rename = "ACTION_ORCHESTRATOR_NEW")]
    OrchestratorNew(OrchestratorAction),
}

impl ActionEvent {
    /// Returns the event "code", the string that represents the event type.
    pub fn code(&self) -> &'static str {
        match self {
            ActionEvent::Changed(_) => "ACTION_CHANGED",
            ActionEvent::Finished(_) => "ACTION_FINISHED",
            ActionEvent::History(_) => "ACTION_HISTORY",
            ActionEvent::Lost(_) => "ACTION_LOST",
            ActionEvent::New(_) => "ACTION_NEW",
            ActionEvent::OrchestratorChanged(_) => "ACTION_ORCHESTRATOR_CHANGED",
            ActionEvent::OrchestratorFinished(_) => "ACTION_ORCHESTRATOR_FINISHED",
            ActionEvent::OrchestratorNew(_) => "ACTION_ORCHESTRATOR_NEW",
        }
    }

    /// Identifier of the cluster the action event is about.
    pub fn entity_id(&self) -> EntityId {
        let cluster_id = match self {
            ActionEvent::Changed(change) => &change.cluster_id,
            ActionEvent::Finished(action) => &action.cluster_id,
            ActionEvent::History(info) => &info.cluster_id,
            ActionEvent::Lost(action) => &action.cluster_id,
            ActionEvent::New(action) => &action.cluster_id,
            ActionEvent::OrchestratorChanged(action) => &action.current.cluster_id,
            ActionEvent::OrchestratorFinished(action) => &action.cluster_id,
            ActionEvent::OrchestratorNew(action) => &action.cluster_id,
        };
        let node = match self {
            ActionEvent::Changed(change) => Some(&change.current.node_id),
            ActionEvent::Finished(action) => Some(&action.node_id),
            ActionEvent::History(info) => Some(&info.node_id),
            ActionEvent::Lost(action) => Some(&action.node_id),
            ActionEvent::New(action) => Some(&action.node_id),
            ActionEvent::OrchestratorChanged(_) => None,
            ActionEvent::OrchestratorFinished(_) => None,
            ActionEvent::OrchestratorNew(_) => None,
        };
        let action = match self {
            ActionEvent::Changed(change) => change.current.action_id,
            ActionEvent::Finished(action) => action.action_id,
            ActionEvent::History(info) => info.action_id,
            ActionEvent::Lost(action) => action.action_id,
            ActionEvent::New(action) => action.action_id,
            ActionEvent::OrchestratorChanged(action) => action.current.action_id,
            ActionEvent::OrchestratorFinished(action) => action.action_id,
            ActionEvent::OrchestratorNew(action) => action.action_id,
        };
        // TODO: Must use a static string because of refs until actions have namespaces attached.
        let _ns = Namespace::HARDCODED_FOR_ROLLOUT();
        match node {
            Some(node) => EntityId::NodeAction("default", cluster_id, node, action),
            None => EntityId::OrchestratorAction("default", cluster_id, action),
        }
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

    /// Build an `ActionEvent::OrchestratorNew` event.
    pub fn new_orchestrator_action(self, action: OrchestratorAction) -> Event {
        let event = ActionEvent::OrchestratorNew(action);
        let payload = Payload::Action(event);
        self.builder.finish(payload)
    }

    /// Build an `ActionEvent::OrchestratorChanged` event.
    pub fn orchestrator_action_changed(
        self,
        old: OrchestratorAction,
        new: OrchestratorAction,
    ) -> Event {
        let event = Box::new(OrchestratorActionChanged {
            current: new,
            previous: old,
        });
        let event = ActionEvent::OrchestratorChanged(event);
        let payload = Payload::Action(event);
        self.builder.finish(payload)
    }

    /// Build an `ActionEvent::OrchestratorFinished` event.
    pub fn orchestrator_action_finished(self, action: OrchestratorAction) -> Event {
        let event = ActionEvent::OrchestratorFinished(action);
        let payload = Payload::Action(event);
        self.builder.finish(payload)
    }
}

/// Hold data about an action change with before and after state.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct OrchestratorActionChanged {
    pub current: OrchestratorAction,
    pub previous: OrchestratorAction,
}

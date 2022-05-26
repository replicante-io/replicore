use serde::Deserialize;
use serde::Serialize;

use super::node::NodeEvent;
use super::Event;
use super::EventBuilder;
use super::Payload;
use crate::agent::Agent;
use crate::agent::AgentInfo;
use crate::agent::AgentStatus;
use crate::scope::EntityId;
use crate::scope::Namespace;

/// Enumerates all possible agent events emitted by the system.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
// TODO: use when possible #[non_exhaustive]
pub enum AgentEvent {
    /// An agent was found to be down.
    #[serde(rename = "AGENT_DOWN")]
    Down(StatusChange),

    /// Information about an agent changed.
    #[serde(rename = "AGENT_INFO_CHANGED")]
    InfoChanged(InfoChanged),

    /// Information about an agent was collected for the first time.
    #[serde(rename = "AGENT_INFO_NEW")]
    InfoNew(AgentInfo),

    /// An agent was discovered for the first time.
    #[serde(rename = "AGENT_NEW")]
    New(Agent),

    /// An agent was found to be up.
    #[serde(rename = "AGENT_UP")]
    Up(StatusChange),
}

impl AgentEvent {
    /// Returns the event "code", the string that represents the event type.
    pub fn code(&self) -> &'static str {
        match self {
            AgentEvent::Down(_) => "AGENT_DOWN",
            AgentEvent::InfoChanged(_) => "AGENT_INFO_CHANGED",
            AgentEvent::InfoNew(_) => "AGENT_INFO_NEW",
            AgentEvent::New(_) => "AGENT_NEW",
            AgentEvent::Up(_) => "AGENT_UP",
        }
    }

    /// Identifier of the node the event is about.
    pub fn entity_id(&self) -> EntityId {
        let cluster_id = match self {
            AgentEvent::Down(change) => &change.cluster_id,
            AgentEvent::InfoChanged(change) => &change.cluster_id,
            AgentEvent::InfoNew(info) => &info.cluster_id,
            AgentEvent::New(agent) => &agent.cluster_id,
            AgentEvent::Up(change) => &change.cluster_id,
        };
        let node = match self {
            AgentEvent::Down(change) => &change.host,
            AgentEvent::InfoChanged(change) => &change.after.host,
            AgentEvent::InfoNew(info) => &info.host,
            AgentEvent::New(agent) => &agent.host,
            AgentEvent::Up(change) => &change.host,
        };
        // TODO: Must use a static string because of refs until agents have namespaces attached.
        let _ns = Namespace::HARDCODED_FOR_ROLLOUT();
        EntityId::Node("default", cluster_id, node)
    }
}

/// Build `AgentEvent`s, validating inputs.
pub struct AgentEventBuilder {
    pub(super) builder: EventBuilder,
}

impl AgentEventBuilder {
    /// Build an `AgentEvent::InfoChanged` event.
    pub fn info_changed(self, before: AgentInfo, after: AgentInfo) -> Event {
        let event = AgentEvent::InfoChanged(InfoChanged {
            after,
            cluster_id: before.cluster_id.clone(),
            before,
        });
        let payload = Payload::Agent(event);
        self.builder.finish(payload)
    }

    /// Build an `AgentEvent::New` event.
    pub fn new_agent(self, agent: Agent) -> Event {
        let event = AgentEvent::New(agent);
        let payload = Payload::Agent(event);
        self.builder.finish(payload)
    }

    /// Build an `AgentEvent::InfoNew` event.
    pub fn new_agent_info(self, agent: AgentInfo) -> Event {
        let event = AgentEvent::InfoNew(agent);
        let payload = Payload::Agent(event);
        self.builder.finish(payload)
    }

    /// Build an agent status transition event.
    ///
    /// This method may return non-`AGENT_*` events when datastores change state.
    ///
    /// # Transition table
    /// ```text
    ///       \ To | Up        | Node Down | Agent Down
    ///   From \   |           |           |
    /// -----------|-----------|-----------|-----------
    /// Agent Down | Up        | Node Down | Agent Down
    /// Node Down  | Node Up   | Node Down | Agent Down
    /// Up         | Up (NoOp) | Node Down | Agent Down
    /// ```
    pub fn transition(self, before: Agent, after: Agent) -> Event {
        let payload = match (before.status, after.status) {
            (AgentStatus::AgentDown(data), new_status) => self.payload_from_after(
                before.cluster_id,
                before.host,
                AgentStatus::AgentDown(data),
                new_status,
            ),
            (AgentStatus::NodeDown(data), AgentStatus::Up) => {
                let event = NodeEvent::Up(StatusChange {
                    after: AgentStatus::Up,
                    before: AgentStatus::NodeDown(data),
                    cluster_id: before.cluster_id,
                    host: before.host,
                });
                Payload::Node(event)
            }
            (AgentStatus::NodeDown(data), new_status) => self.payload_from_after(
                before.cluster_id,
                before.host,
                AgentStatus::NodeDown(data),
                new_status,
            ),
            (AgentStatus::Up, AgentStatus::Up) => {
                let event = AgentEvent::Up(StatusChange {
                    after: AgentStatus::Up,
                    before: AgentStatus::Up,
                    cluster_id: before.cluster_id,
                    host: before.host,
                });
                Payload::Agent(event)
            }
            (AgentStatus::Up, new_status) => {
                self.payload_from_after(before.cluster_id, before.host, AgentStatus::Up, new_status)
            }
        };
        self.builder.finish(payload)
    }

    /// Generate an event payload based on the status we are transitioning to.
    fn payload_from_after(
        &self,
        cluster_id: String,
        host: String,
        before: AgentStatus,
        after: AgentStatus,
    ) -> Payload {
        match after {
            AgentStatus::AgentDown(_) => {
                let event = AgentEvent::Down(StatusChange {
                    after,
                    before,
                    cluster_id,
                    host,
                });
                Payload::Agent(event)
            }
            AgentStatus::NodeDown(_) => {
                let event = NodeEvent::Down(StatusChange {
                    after,
                    before,
                    cluster_id,
                    host,
                });
                Payload::Node(event)
            }
            AgentStatus::Up => {
                let event = AgentEvent::Up(StatusChange {
                    after,
                    before,
                    cluster_id,
                    host,
                });
                Payload::Agent(event)
            }
        }
    }
}

/// Metadata attached to agent info changed.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct InfoChanged {
    pub after: AgentInfo,
    pub before: AgentInfo,
    pub cluster_id: String,
}

/// Metadata attached to agent status change events.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct StatusChange {
    pub after: AgentStatus,
    pub before: AgentStatus,
    pub cluster_id: String,
    pub host: String,
}

#[cfg(test)]
mod tests {
    use super::AgentEvent;
    use super::Event;
    use super::InfoChanged;
    use super::NodeEvent;
    use super::Payload;
    use super::StatusChange;
    use crate::agent::Agent;
    use crate::agent::AgentInfo;
    use crate::agent::AgentStatus;

    #[test]
    fn info_changed() {
        let after = AgentInfo {
            cluster_id: "cluster".into(),
            host: "host".into(),
            version_checkout: "abc".into(),
            version_number: "1.2.3".into(),
            version_taint: "tainted".into(),
        };
        let before = AgentInfo {
            cluster_id: "cluster".into(),
            host: "host".into(),
            version_checkout: "abc".into(),
            version_number: "1.2.3".into(),
            version_taint: "none".into(),
        };
        let event = Event::builder()
            .agent()
            .info_changed(before.clone(), after.clone());
        let expected = Payload::Agent(AgentEvent::InfoChanged(InfoChanged {
            after,
            before,
            cluster_id: "cluster".into(),
        }));
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn new_agent() {
        let agent = Agent::new("cluster", "host", AgentStatus::Up);
        let event = Event::builder().agent().new_agent(agent.clone());
        let expected = Payload::Agent(AgentEvent::New(agent));
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn new_agent_info() {
        let agent = AgentInfo {
            cluster_id: "cluster".into(),
            host: "host".into(),
            version_checkout: "abc".into(),
            version_number: "1.2.3".into(),
            version_taint: "none".into(),
        };
        let event = Event::builder().agent().new_agent_info(agent.clone());
        let expected = Payload::Agent(AgentEvent::InfoNew(agent));
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn transition_agent_down_to_agent_down() {
        let after = Agent::new("cluster", "host", AgentStatus::AgentDown("after".into()));
        let before = Agent::new("cluster", "host", AgentStatus::AgentDown("before".into()));
        let event = Event::builder()
            .agent()
            .transition(before.clone(), after.clone());
        let expected = Payload::Agent(AgentEvent::Down(StatusChange {
            cluster_id: "cluster".into(),
            host: "host".into(),
            after: after.status,
            before: before.status,
        }));
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn transition_agent_down_to_datastore_down() {
        let after = Agent::new("cluster", "host", AgentStatus::NodeDown("after".into()));
        let before = Agent::new("cluster", "host", AgentStatus::AgentDown("before".into()));
        let event = Event::builder()
            .agent()
            .transition(before.clone(), after.clone());
        let expected = Payload::Node(NodeEvent::Down(StatusChange {
            cluster_id: "cluster".into(),
            host: "host".into(),
            after: after.status,
            before: before.status,
        }));
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn transition_agent_down_to_up() {
        let after = Agent::new("cluster", "host", AgentStatus::Up);
        let before = Agent::new("cluster", "host", AgentStatus::AgentDown("before".into()));
        let event = Event::builder()
            .agent()
            .transition(before.clone(), after.clone());
        let expected = Payload::Agent(AgentEvent::Up(StatusChange {
            cluster_id: "cluster".into(),
            host: "host".into(),
            after: after.status,
            before: before.status,
        }));
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn transition_datastore_down_to_agent_down() {
        let after = Agent::new("cluster", "host", AgentStatus::AgentDown("after".into()));
        let before = Agent::new("cluster", "host", AgentStatus::NodeDown("before".into()));
        let event = Event::builder()
            .agent()
            .transition(before.clone(), after.clone());
        let expected = Payload::Agent(AgentEvent::Down(StatusChange {
            cluster_id: "cluster".into(),
            host: "host".into(),
            after: after.status,
            before: before.status,
        }));
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn transition_datastore_down_to_datastore_down() {
        let after = Agent::new("cluster", "host", AgentStatus::NodeDown("after".into()));
        let before = Agent::new("cluster", "host", AgentStatus::NodeDown("before".into()));
        let event = Event::builder()
            .agent()
            .transition(before.clone(), after.clone());
        let expected = Payload::Node(NodeEvent::Down(StatusChange {
            cluster_id: "cluster".into(),
            host: "host".into(),
            after: after.status,
            before: before.status,
        }));
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn transition_datastore_down_to_up() {
        let after = Agent::new("cluster", "host", AgentStatus::Up);
        let before = Agent::new("cluster", "host", AgentStatus::NodeDown("before".into()));
        let event = Event::builder()
            .agent()
            .transition(before.clone(), after.clone());
        let expected = Payload::Node(NodeEvent::Up(StatusChange {
            cluster_id: "cluster".into(),
            host: "host".into(),
            after: after.status,
            before: before.status,
        }));
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn transition_up_to_agent_down() {
        let after = Agent::new("cluster", "host", AgentStatus::AgentDown("TEST".into()));
        let before = Agent::new("cluster", "host", AgentStatus::Up);
        let event = Event::builder()
            .agent()
            .transition(before.clone(), after.clone());
        let expected = Payload::Agent(AgentEvent::Down(StatusChange {
            cluster_id: "cluster".into(),
            host: "host".into(),
            after: after.status,
            before: before.status,
        }));
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn transition_up_to_datastore_down() {
        let after = Agent::new("cluster", "host", AgentStatus::NodeDown("TEST".into()));
        let before = Agent::new("cluster", "host", AgentStatus::Up);
        let event = Event::builder()
            .agent()
            .transition(before.clone(), after.clone());
        let expected = Payload::Node(NodeEvent::Down(StatusChange {
            cluster_id: "cluster".into(),
            host: "host".into(),
            after: after.status,
            before: before.status,
        }));
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn transition_up_to_up() {
        let after = Agent::new("cluster", "host", AgentStatus::Up);
        let before = Agent::new("cluster", "host", AgentStatus::Up);
        let event = Event::builder()
            .agent()
            .transition(before.clone(), after.clone());
        let expected = Payload::Agent(AgentEvent::Up(StatusChange {
            cluster_id: "cluster".into(),
            host: "host".into(),
            after: after.status,
            before: before.status,
        }));
        assert_eq!(event.payload, expected);
    }
}

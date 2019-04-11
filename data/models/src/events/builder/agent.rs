use super::super::super::Agent;
use super::super::super::AgentInfo;
use super::super::super::AgentStatus;

use super::super::AgentNew;
use super::super::AgentInfoChanged;
use super::super::AgentStatusChange;
use super::super::Event;
use super::super::EventBuilder;
use super::super::EventPayload;

/// Build `Event`s that belongs to the agnet family.
pub struct AgentBuilder {
    builder: EventBuilder,
}

impl AgentBuilder {
    /// Create a new agent event builder.
    pub fn builder(builder: EventBuilder) -> AgentBuilder {
        AgentBuilder { builder }
    }

    /// Build an `EventPayload::AgentNew`.
    pub fn agent_new(self, cluster_id: String, host: String) -> Event {
        let data = EventPayload::AgentNew(AgentNew { cluster_id, host });
        self.builder.build(data)
    }

    /// Specialise the builder into an agent info event builder.
    pub fn info(self) -> AgentInfoBuilder {
        AgentInfoBuilder::builder(self.builder)
    }

    /// Build an agent status transition event.
    ///
    /// # Transition table
    /// ```text
    ///       \ To | Up       | Node Down | Agent Down
    ///   From \   |          |           |
    /// -----------|----------|-----------|-----------
    /// Agent Down | Up       | Node Down | Agent Down
    /// Node Down  | Node Up  | Node Down | Agent Down
    /// Up         | Panic    | Node Down | Agent Down
    /// ```
    ///
    /// # Panics
    /// This method panics if the agent goes from UP to UP.
    pub fn transition(self, before: Agent, after: Agent) -> Event {
        let data = match (before.status, after.status) {
            (AgentStatus::AgentDown(data), new_status) => self.payload_from_after(
                before.cluster_id, before.host, AgentStatus::AgentDown(data), new_status
            ),
            (AgentStatus::NodeDown(data), AgentStatus::Up) => EventPayload::NodeUp(AgentStatusChange {
                cluster_id: before.cluster_id,
                host: before.host,
                before: AgentStatus::NodeDown(data),
                after: AgentStatus::Up,
            }),
            (AgentStatus::NodeDown(data), new_status) => self.payload_from_after(
                before.cluster_id, before.host, AgentStatus::NodeDown(data), new_status
            ),
            (AgentStatus::Up, AgentStatus::Up) => panic!("An agent can't go from UP to UP"),
            (AgentStatus::Up, new_status) => self.payload_from_after(
                before.cluster_id, before.host, AgentStatus::Up, new_status
            ),
        };
        self.builder.build(data)
    }
}

impl AgentBuilder {
    /// Generate an event payload based on the status we are transitioning to.
    fn payload_from_after(
        &self, cluster_id: String, host: String, before: AgentStatus, after: AgentStatus
    ) -> EventPayload {
        match after {
            AgentStatus::AgentDown(_) => EventPayload::AgentDown(AgentStatusChange {
                cluster_id,
                host,
                before,
                after,
            }),
            AgentStatus::NodeDown(_) => EventPayload::NodeDown(AgentStatusChange {
                cluster_id,
                host,
                before,
                after,
            }),
            AgentStatus::Up => EventPayload::AgentUp(AgentStatusChange {
                cluster_id,
                host,
                before,
                after,
            }),
        }
    }
}

/// Build `Event`s that belongs to the agnet info family.
pub struct AgentInfoBuilder {
    builder: EventBuilder,
}

impl AgentInfoBuilder {
    /// Create a new agent info event builder.
    pub fn builder(builder: EventBuilder) -> AgentInfoBuilder {
        AgentInfoBuilder { builder }
    }

    /// Build an `EventPayload::AgentInfoChanged` event.
    pub fn changed(self, before: AgentInfo, after: AgentInfo) -> Event {
        let data = EventPayload::AgentInfoChanged(AgentInfoChanged {
            cluster_id: before.cluster_id.clone(),
            before,
            after,
        });
        self.builder.build(data)
    }

    /// Build an `EventPayload::AgentInfoNew` event.
    pub fn info_new(self, agent: AgentInfo) -> Event {
        let data = EventPayload::AgentInfoNew(agent);
        self.builder.build(data)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::super::AgentStatus;
    use super::super::super::AgentNew;
    use super::Agent;
    use super::Event;
    use super::EventPayload;

    #[test]
    fn new() {
        let event = Event::builder().agent().agent_new("cluster".into(), "host".into());
        let expected = EventPayload::AgentNew(AgentNew {
            cluster_id: "cluster".into(),
            host: "host".into(),
        });
        assert_eq!(event.payload, expected);
    }

    mod transition {
        use super::super::AgentStatusChange;
        use super::Agent;
        use super::AgentStatus;
        use super::Event;
        use super::EventPayload;

        #[test]
        fn agent_down_to_agent_down() {
            let after = Agent::new("cluster", "host", AgentStatus::AgentDown("after".into()));
            let before = Agent::new("cluster", "host", AgentStatus::AgentDown("before".into()));
            let event = Event::builder().agent().transition(before.clone(), after.clone());
            let expected = EventPayload::AgentDown(AgentStatusChange {
                cluster_id: "cluster".into(),
                host: "host".into(),
                after: after.status,
                before: before.status,
            });
            assert_eq!(event.payload, expected);
        }

        #[test]
        fn agent_down_to_datastore_down() {
            let after = Agent::new("cluster", "host", AgentStatus::NodeDown("after".into()));
            let before = Agent::new("cluster", "host", AgentStatus::AgentDown("before".into()));
            let event = Event::builder().agent().transition(before.clone(), after.clone());
            let expected = EventPayload::NodeDown(AgentStatusChange {
                cluster_id: "cluster".into(),
                host: "host".into(),
                after: after.status,
                before: before.status,
            });
            assert_eq!(event.payload, expected);
        }

        #[test]
        fn agent_down_to_up() {
            let after = Agent::new("cluster", "host", AgentStatus::Up);
            let before = Agent::new("cluster", "host", AgentStatus::AgentDown("before".into()));
            let event = Event::builder().agent().transition(before.clone(), after.clone());
            let expected = EventPayload::AgentUp(AgentStatusChange {
                cluster_id: "cluster".into(),
                host: "host".into(),
                after: after.status,
                before: before.status,
            });
            assert_eq!(event.payload, expected);
        }

        #[test]
        fn datastore_down_to_agent_down() {
            let after = Agent::new("cluster", "host", AgentStatus::AgentDown("after".into()));
            let before = Agent::new("cluster", "host", AgentStatus::NodeDown("before".into()));
            let event = Event::builder().agent().transition(before.clone(), after.clone());
            let expected = EventPayload::AgentDown(AgentStatusChange {
                cluster_id: "cluster".into(),
                host: "host".into(),
                after: after.status,
                before: before.status,
            });
            assert_eq!(event.payload, expected);
        }

        #[test]
        fn datastore_down_to_datastore_down() {
            let after = Agent::new("cluster", "host", AgentStatus::NodeDown("after".into()));
            let before = Agent::new("cluster", "host", AgentStatus::NodeDown("before".into()));
            let event = Event::builder().agent().transition(before.clone(), after.clone());
            let expected = EventPayload::NodeDown(AgentStatusChange {
                cluster_id: "cluster".into(),
                host: "host".into(),
                after: after.status,
                before: before.status,
            });
            assert_eq!(event.payload, expected);
        }

        #[test]
        fn datastore_down_to_up() {
            let after = Agent::new("cluster", "host", AgentStatus::Up);
            let before = Agent::new("cluster", "host", AgentStatus::NodeDown("before".into()));
            let event = Event::builder().agent().transition(before.clone(), after.clone());
            let expected = EventPayload::NodeUp(AgentStatusChange {
                cluster_id: "cluster".into(),
                host: "host".into(),
                after: after.status,
                before: before.status,
            });
            assert_eq!(event.payload, expected);
        }

        #[test]
        fn up_to_agent_down() {
            let after = Agent::new("cluster", "host", AgentStatus::AgentDown("TEST".into()));
            let before = Agent::new("cluster", "host", AgentStatus::Up);
            let event = Event::builder().agent().transition(before.clone(), after.clone());
            let expected = EventPayload::AgentDown(AgentStatusChange {
                cluster_id: "cluster".into(),
                host: "host".into(),
                after: after.status,
                before: before.status,
            });
            assert_eq!(event.payload, expected);
        }

        #[test]
        fn up_to_datastore_down() {
            let after = Agent::new("cluster", "host", AgentStatus::NodeDown("TEST".into()));
            let before = Agent::new("cluster", "host", AgentStatus::Up);
            let event = Event::builder().agent().transition(before.clone(), after.clone());
            let expected = EventPayload::NodeDown(AgentStatusChange {
                cluster_id: "cluster".into(),
                host: "host".into(),
                after: after.status,
                before: before.status,
            });
            assert_eq!(event.payload, expected);
        }

        #[test]
        #[should_panic(expected = "An agent can't go from UP to UP")]
        fn up_to_up() {
            let after = Agent::new("cluster", "host", AgentStatus::Up);
            let before = Agent::new("cluster", "host", AgentStatus::Up);
            Event::builder().agent().transition(before, after);
        }
    }

    mod info {
        use replicante_agent_models::AgentInfo as WireAgentInfo;
        use replicante_agent_models::AgentVersion;
        use super::super::super::super::super::AgentInfo;
        use super::super::AgentInfoChanged;
        use super::Event;
        use super::EventPayload;

        #[test]
        fn changed() {
            let before = AgentVersion::new("1.2.3", "abcdef", "tainted");
            let before = WireAgentInfo::new(before);
            let before = AgentInfo::new("cluster", "host", before);
            let after = AgentVersion::new("1.2.3", "abcdef", "tainted");
            let after = WireAgentInfo::new(after);
            let after = AgentInfo::new("cluster", "host", after);
            let event = Event::builder().agent().info().changed(before.clone(), after.clone());
            let expected = EventPayload::AgentInfoChanged(AgentInfoChanged {
                cluster_id: "cluster".into(),
                before,
                after
            });
            assert_eq!(event.payload, expected);
        }

        #[test]
        fn new() {
            let agent = AgentVersion::new("1.2.3", "abcdef", "tainted");
            let agent = WireAgentInfo::new(agent);
            let agent = AgentInfo::new("cluster", "host", agent);
            let event = Event::builder().agent().info().info_new(agent.clone());
            let expected = EventPayload::AgentInfoNew(agent);
            assert_eq!(event.payload, expected);
        }
    }
}

use super::super::super::Agent;
use super::super::super::AgentStatus;

use super::super::AgentStatusChange;
use super::super::Event;
use super::super::EventBuilder;
use super::super::EventData;


/// Build `Event`s that belongs to the agnet family.
pub struct AgentBuilder {
    builder: EventBuilder,
}

impl AgentBuilder {
    /// Create a new agent event builder.
    pub fn builder(builder: EventBuilder) -> AgentBuilder {
        AgentBuilder { builder }
    }

    /// Build an `EventData::AgentNew`.
    pub fn agent_new(self, agent: Agent) -> Event {
        let data = EventData::AgentNew(agent);
        self.builder.build(data)
    }

    /// Build an agent status transition event.
    ///
    /// # Transition table
    /// ```text
    ///     \ To | Up         | DS Down       | A Down
    /// From \   |            |               |
    /// ---------|------------|---------------|--------
    /// Up       | Panic      | DS Down       | A Down
    /// DS Down  | DS Recover | DS Still Down | A Down
    /// A Down   | A Recover  | DS Down       | A Still Down
    /// ```
    ///
    /// # Panics
    /// This method panics if the agent goes from UP to UP.
    pub fn transition(self, before: Agent, after: Agent) -> Event {
        let data = match (&before.status, &after.status) {
            (&AgentStatus::AgentDown(_), &AgentStatus::AgentDown(_)) =>
                EventData::AgentStillDown(AgentStatusChange {
                    cluster: before.cluster,
                    host: before.host,
                    after: after.status,
                    before: before.status,
                }),
            (&AgentStatus::AgentDown(_), &AgentStatus::DatastoreDown(_)) =>
                EventData::DatastoreDown(AgentStatusChange {
                    cluster: before.cluster,
                    host: before.host,
                    after: after.status,
                    before: before.status,
                }),
            (&AgentStatus::AgentDown(_), &AgentStatus::Up) =>
                EventData::AgentRecover(AgentStatusChange {
                    cluster: before.cluster,
                    host: before.host,
                    after: after.status,
                    before: before.status,
                }),
            (&AgentStatus::DatastoreDown(_), &AgentStatus::AgentDown(_)) =>
                EventData::AgentDown(AgentStatusChange {
                    cluster: before.cluster,
                    host: before.host,
                    after: after.status,
                    before: before.status,
                }),
            (&AgentStatus::DatastoreDown(_), &AgentStatus::DatastoreDown(_)) =>
                EventData::DatastoreStillDown(AgentStatusChange {
                    cluster: before.cluster,
                    host: before.host,
                    after: after.status,
                    before: before.status,
                }),
            (&AgentStatus::DatastoreDown(_), &AgentStatus::Up) =>
                EventData::DatastoreRecover(AgentStatusChange {
                    cluster: before.cluster,
                    host: before.host,
                    after: after.status,
                    before: before.status,
                }),
            (&AgentStatus::Up, &AgentStatus::AgentDown(_)) =>
                EventData::AgentDown(AgentStatusChange {
                    cluster: before.cluster,
                    host: before.host,
                    after: after.status,
                    before: before.status,
                }),
            (&AgentStatus::Up, &AgentStatus::DatastoreDown(_)) =>
                EventData::DatastoreDown(AgentStatusChange {
                    cluster: before.cluster,
                    host: before.host,
                    after: after.status,
                    before: before.status,
                }),
            (&AgentStatus::Up, &AgentStatus::Up) => panic!("An agent can't go from UP to UP"),
        };
        self.builder.build(data)
    }
}


#[cfg(test)]
mod tests {
    use super::super::super::super::AgentStatus;
    use super::Agent;
    use super::Event;
    use super::EventData;

    #[test]
    fn new() {
        let agent = Agent::new("cluster", "host", AgentStatus::AgentDown("TEST".into()));
        let event = Event::builder().agent().agent_new(agent.clone());
        let expected = EventData::AgentNew(agent);
        assert_eq!(event.payload, expected);
    }

    mod transition {
        use super::super::AgentStatusChange;
        use super::Agent;
        use super::AgentStatus;
        use super::Event;
        use super::EventData;

        #[test]
        fn agent_down_to_agent_down() {
            let after = Agent::new("cluster", "host", AgentStatus::AgentDown("after".into()));
            let before = Agent::new("cluster", "host", AgentStatus::AgentDown("before".into()));
            let event = Event::builder().agent().transition(before.clone(), after.clone());
            let expected = EventData::AgentStillDown(AgentStatusChange {
                cluster: "cluster".into(),
                host: "host".into(),
                after: after.status,
                before: before.status,
            });
            assert_eq!(event.payload, expected);
        }

        #[test]
        fn agent_down_to_datastore_down() {
            let after = Agent::new("cluster", "host", AgentStatus::DatastoreDown("after".into()));
            let before = Agent::new("cluster", "host", AgentStatus::AgentDown("before".into()));
            let event = Event::builder().agent().transition(before.clone(), after.clone());
            let expected = EventData::DatastoreDown(AgentStatusChange {
                cluster: "cluster".into(),
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
            let expected = EventData::AgentRecover(AgentStatusChange {
                cluster: "cluster".into(),
                host: "host".into(),
                after: after.status,
                before: before.status,
            });
            assert_eq!(event.payload, expected);
        }

        #[test]
        fn datastore_down_to_agent_down() {
            let after = Agent::new("cluster", "host", AgentStatus::AgentDown("after".into()));
            let before = Agent::new("cluster", "host", AgentStatus::DatastoreDown("before".into()));
            let event = Event::builder().agent().transition(before.clone(), after.clone());
            let expected = EventData::AgentDown(AgentStatusChange {
                cluster: "cluster".into(),
                host: "host".into(),
                after: after.status,
                before: before.status,
            });
            assert_eq!(event.payload, expected);
        }

        #[test]
        fn datastore_down_to_datastore_down() {
            let after = Agent::new("cluster", "host", AgentStatus::DatastoreDown("after".into()));
            let before = Agent::new("cluster", "host", AgentStatus::DatastoreDown("before".into()));
            let event = Event::builder().agent().transition(before.clone(), after.clone());
            let expected = EventData::DatastoreStillDown(AgentStatusChange {
                cluster: "cluster".into(),
                host: "host".into(),
                after: after.status,
                before: before.status,
            });
            assert_eq!(event.payload, expected);
        }

        #[test]
        fn datastore_down_to_up() {
            let after = Agent::new("cluster", "host", AgentStatus::Up);
            let before = Agent::new("cluster", "host", AgentStatus::DatastoreDown("before".into()));
            let event = Event::builder().agent().transition(before.clone(), after.clone());
            let expected = EventData::DatastoreRecover(AgentStatusChange {
                cluster: "cluster".into(),
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
            let expected = EventData::AgentDown(AgentStatusChange {
                cluster: "cluster".into(),
                host: "host".into(),
                after: after.status,
                before: before.status,
            });
            assert_eq!(event.payload, expected);
        }

        #[test]
        fn up_to_datastore_down() {
            let after = Agent::new("cluster", "host", AgentStatus::DatastoreDown("TEST".into()));
            let before = Agent::new("cluster", "host", AgentStatus::Up);
            let event = Event::builder().agent().transition(before.clone(), after.clone());
            let expected = EventData::DatastoreDown(AgentStatusChange {
                cluster: "cluster".into(),
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
}

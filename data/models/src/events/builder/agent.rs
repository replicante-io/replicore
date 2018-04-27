use super::super::super::Agent;
use super::super::Event;
use super::super::EventBuilder;
use super::super::EventData;


/// Build `Event`s that belongs to the agnet family.
pub struct AgentBuilder {
    builder: EventBuilder,
}

impl AgentBuilder {
    /// Create a new agent event builder.
    pub fn new_builder(builder: EventBuilder) -> AgentBuilder {
        AgentBuilder { builder }
    }

    /// Build an `EventData::AgentNew`.
    pub fn new(self, agent: Agent) -> Event {
        let data = EventData::AgentNew(agent);
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
    fn agent_new() {
        let agent = Agent::new("cluster", "host", AgentStatus::AgentDown("TEST".into()));
        let event = Event::builder().agent().new(agent.clone());
        let expected = EventData::AgentNew(agent);
        assert_eq!(event.event, expected);
    }
}

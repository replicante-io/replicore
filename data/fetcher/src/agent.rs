use failure::ResultExt;

use replicante_agent_client::Client;
use replicante_data_models::Agent;
use replicante_data_models::AgentInfo;
use replicante_data_models::AgentStatus;
use replicante_data_models::Event;
use replicante_data_store::store::Store;
use replicante_streams_events::EventsStream;

use super::Error;
use super::ErrorKind;
use super::Result;

/// Subset of fetcher logic that deals specifically with agents.
pub(crate) struct AgentFetcher {
    events: EventsStream,
    store: Store,
}

impl AgentFetcher {
    pub(crate) fn new(events: EventsStream, store: Store) -> AgentFetcher {
        AgentFetcher { events, store }
    }

    pub(crate) fn process_agent(&self, agent: Agent) -> Result<()> {
        let old = self
            .store
            .agent(agent.cluster_id.clone(), agent.host.clone())
            .get();
        match old {
            Err(error) => Err(error)
                .with_context(|_| ErrorKind::StoreRead("agent"))
                .map_err(Error::from),
            Ok(None) => self.process_agent_new(agent),
            Ok(Some(old)) => self.process_agent_existing(agent, old),
        }
    }

    pub(crate) fn process_agent_info(
        &self,
        client: &Client,
        cluster_id: String,
        node: String,
    ) -> Result<()> {
        let info = client
            .agent_info()
            .with_context(|_| ErrorKind::AgentRead("agent info", client.id().to_string()))?;
        let info = AgentInfo::new(cluster_id, node, info);
        let old = self
            .store
            .agent(info.cluster_id.clone(), info.host.clone())
            .info();
        match old {
            Err(error) => Err(error)
                .with_context(|_| ErrorKind::StoreRead("agent info"))
                .map_err(Error::from),
            Ok(None) => self.process_agent_info_new(info),
            Ok(Some(old)) => self.process_agent_info_existing(info, old),
        }
    }
}

impl AgentFetcher {
    fn process_agent_existing(&self, agent: Agent, old: Agent) -> Result<()> {
        if agent == old {
            return Ok(());
        }
        if agent.status != old.status {
            let event = Event::builder().agent().transition(old, agent.clone());
            let code = event.code();
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::EventEmit(code))?;
        }
        self.store
            .persist()
            .agent(agent)
            .with_context(|_| ErrorKind::StoreWrite("agent update"))
            .map_err(Error::from)
    }

    fn process_agent_new(&self, agent: Agent) -> Result<()> {
        let event = Event::builder()
            .agent()
            .agent_new(agent.cluster_id.clone(), agent.host.clone());
        let code = event.code();
        self.events
            .emit(event)
            .with_context(|_| ErrorKind::EventEmit(code))?;

        // Emit a synthetic transition to up.
        let before = AgentStatus::AgentDown("Newly discovered agent".into());
        let before = Agent::new(agent.cluster_id.clone(), agent.host.clone(), before);
        let event = Event::builder().agent().transition(before, agent.clone());
        let code = event.code();
        self.events
            .emit(event)
            .with_context(|_| ErrorKind::EventEmit(code))?;
        self.store
            .persist()
            .agent(agent)
            .with_context(|_| ErrorKind::StoreWrite("new agent"))
            .map_err(Error::from)
    }

    fn process_agent_info_existing(&self, agent: AgentInfo, old: AgentInfo) -> Result<()> {
        if agent != old {
            let event = Event::builder().agent().info().changed(old, agent.clone());
            let code = event.code();
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::EventEmit(code))?;
        }
        // ALWAYS persist the model, even unchanged, to clear the staleness state.
        self.store
            .persist()
            .agent_info(agent)
            .with_context(|_| ErrorKind::StoreWrite("agent info update"))
            .map_err(Error::from)
    }

    fn process_agent_info_new(&self, agent: AgentInfo) -> Result<()> {
        let event = Event::builder().agent().info().info_new(agent.clone());
        let code = event.code();
        self.events
            .emit(event)
            .with_context(|_| ErrorKind::EventEmit(code))?;
        self.store
            .persist()
            .agent_info(agent)
            .with_context(|_| ErrorKind::StoreWrite("new agent info"))
            .map_err(Error::from)
    }
}

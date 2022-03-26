use failure::ResultExt;
use opentracingrust::Span;

use replicante_agent_client::Client;
use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::AgentStatus;
use replicante_models_core::events::Event;
use replicante_store_primary::store::Store;
use replicante_stream_events::EmitMessage;
use replicante_stream_events::Stream as EventsStream;

use replicore_cluster_view::ClusterView;

use crate::Error;
use crate::ErrorKind;
use crate::Result;

/// Subset of fetcher logic that deals specifically with agents.
pub(crate) struct AgentFetcher {
    events: EventsStream,
    store: Store,
}

impl AgentFetcher {
    pub(crate) fn new(events: EventsStream, store: Store) -> AgentFetcher {
        AgentFetcher { events, store }
    }

    pub(crate) fn process_agent(
        &self,
        cluster_view: &ClusterView,
        agent: Agent,
        span: &mut Span,
    ) -> Result<()> {
        let old = cluster_view.agents.get(&agent.host).cloned();
        match old {
            None => self.process_agent_new(agent, span),
            Some(old) => self.process_agent_existing(agent, old, span),
        }
    }

    pub(crate) fn process_agent_info(
        &self,
        client: &dyn Client,
        cluster_view: &ClusterView,
        node: String,
        span: &mut Span,
    ) -> Result<()> {
        let info = client
            .agent_info(span.context().clone().into())
            .with_context(|_| ErrorKind::AgentDown("agent info", client.id().to_string()))?;
        let info = AgentInfo::new(cluster_view.cluster_id.clone(), node, info);
        let old = cluster_view.agents_info.get(&info.host).cloned();
        match old {
            None => self.process_agent_info_new(info, span),
            Some(old) => self.process_agent_info_existing(info, old, span),
        }
    }
}

impl AgentFetcher {
    fn process_agent_existing(&self, agent: Agent, old: Agent, span: &mut Span) -> Result<()> {
        if agent == old {
            return Ok(());
        }
        if agent.status != old.status {
            let event = Event::builder().agent().transition(old, agent.clone());
            let code = event.code();
            let stream_key = event.stream_key();
            let event = EmitMessage::with(stream_key, event)
                .with_context(|_| ErrorKind::EventEmit(code))?
                .trace(span.context().clone());
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::EventEmit(code))?;
        }
        self.store
            .persist()
            .agent(agent, span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreWrite("agent update"))
            .map_err(Error::from)
    }

    fn process_agent_new(&self, agent: Agent, span: &mut Span) -> Result<()> {
        let event = Event::builder().agent().new_agent(agent.clone());
        let code = event.code();
        let stream_key = event.stream_key();
        let event = EmitMessage::with(stream_key, event)
            .with_context(|_| ErrorKind::EventEmit(code))?
            .trace(span.context().clone());
        self.events
            .emit(event)
            .with_context(|_| ErrorKind::EventEmit(code))?;

        // Emit a synthetic transition to up.
        let before = AgentStatus::AgentDown("Newly discovered agent".into());
        let before = Agent::new(agent.cluster_id.clone(), agent.host.clone(), before);
        let event = Event::builder().agent().transition(before, agent.clone());
        let code = event.code();
        let stream_key = event.stream_key();
        let event = EmitMessage::with(stream_key, event)
            .with_context(|_| ErrorKind::EventEmit(code))?
            .trace(span.context().clone());
        self.events
            .emit(event)
            .with_context(|_| ErrorKind::EventEmit(code))?;
        self.store
            .persist()
            .agent(agent, span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreWrite("new agent"))
            .map_err(Error::from)
    }

    fn process_agent_info_existing(
        &self,
        agent: AgentInfo,
        old: AgentInfo,
        span: &mut Span,
    ) -> Result<()> {
        if agent != old {
            let event = Event::builder().agent().info_changed(old, agent.clone());
            let code = event.code();
            let stream_key = event.stream_key();
            let event = EmitMessage::with(stream_key, event)
                .with_context(|_| ErrorKind::EventEmit(code))?
                .trace(span.context().clone());
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::EventEmit(code))?;
        }
        // ALWAYS persist the model, even unchanged, to clear the staleness state.
        self.store
            .persist()
            .agent_info(agent, span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreWrite("agent info update"))
            .map_err(Error::from)
    }

    fn process_agent_info_new(&self, agent: AgentInfo, span: &mut Span) -> Result<()> {
        let event = Event::builder().agent().new_agent_info(agent.clone());
        let code = event.code();
        let stream_key = event.stream_key();
        let event = EmitMessage::with(stream_key, event)
            .with_context(|_| ErrorKind::EventEmit(code))?
            .trace(span.context().clone());
        self.events
            .emit(event)
            .with_context(|_| ErrorKind::EventEmit(code))?;
        self.store
            .persist()
            .agent_info(agent, span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreWrite("new agent info"))
            .map_err(Error::from)
    }
}

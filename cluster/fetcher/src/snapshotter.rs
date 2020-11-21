use failure::ResultExt;
use opentracingrust::Span;

use replicante_models_core::events::Event;
use replicante_store_primary::store::Store;
use replicante_stream_events::EmitMessage;
use replicante_stream_events::Stream as EventsStream;

use super::ErrorKind;
use super::Result;

/// Emits snapshots for the states of a cluster.
pub struct Snapshotter {
    cluster: String,
    events: EventsStream,
    store: Store,
}

impl Snapshotter {
    pub fn new(cluster: String, events: EventsStream, store: Store) -> Snapshotter {
        Snapshotter {
            cluster,
            events,
            store,
        }
    }

    pub fn run(&self, span: &mut Span) -> Result<()> {
        self.discovery(span)?;
        self.agents(span)?;
        self.nodes(span)?;
        self.shards(span)?;
        Ok(())
    }
}

impl Snapshotter {
    fn agents(&self, span: &mut Span) -> Result<()> {
        let statuses = self
            .store
            .agents(self.cluster.clone())
            .iter(span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreRead("agents statuses"))?;
        for status in statuses {
            let status = status.with_context(|_| ErrorKind::PrimaryStoreRead("agent status"))?;
            let event = Event::builder().snapshot().agent(status);
            let code = event.code();
            let stream_key = event.stream_key();
            let event = EmitMessage::with(stream_key, event)
                .with_context(|_| ErrorKind::EventEmit(code))?
                .trace(span.context().clone());
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::EventEmit(code))?;
        }
        let infos = self
            .store
            .agents(self.cluster.clone())
            .iter_info(span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreRead("agents info"))?;
        for info in infos {
            let info = info.with_context(|_| ErrorKind::PrimaryStoreRead("agent info"))?;
            let event = Event::builder().snapshot().agent_info(info);
            let code = event.code();
            let stream_key = event.stream_key();
            let event = EmitMessage::with(stream_key, event)
                .with_context(|_| ErrorKind::EventEmit(code))?
                .trace(span.context().clone());
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::EventEmit(code))?;
        }
        Ok(())
    }

    fn discovery(&self, span: &mut Span) -> Result<()> {
        let discovery = self
            .store
            .cluster("TODO_NS_NAME".to_string(), self.cluster.clone())
            .discovery(span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreRead("discovery"))?;
        if let Some(discovery) = discovery {
            let event = Event::builder().snapshot().discovery(discovery);
            let code = event.code();
            let stream_key = event.stream_key();
            let event = EmitMessage::with(stream_key, event)
                .with_context(|_| ErrorKind::EventEmit(code))?
                .trace(span.context().clone());
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::EventEmit(code))?;
        }
        Ok(())
    }

    fn nodes(&self, span: &mut Span) -> Result<()> {
        let nodes = self
            .store
            .nodes(self.cluster.clone())
            .iter(span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreRead("nodes"))?;
        for node in nodes {
            let node = node.with_context(|_| ErrorKind::PrimaryStoreRead("node"))?;
            let event = Event::builder().snapshot().node(node);
            let code = event.code();
            let stream_key = event.stream_key();
            let event = EmitMessage::with(stream_key, event)
                .with_context(|_| ErrorKind::EventEmit(code))?
                .trace(span.context().clone());
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::EventEmit(code))?;
        }
        Ok(())
    }

    fn shards(&self, span: &mut Span) -> Result<()> {
        let shards = self
            .store
            .shards(self.cluster.clone())
            .iter(span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreRead("shards"))?;
        for shard in shards {
            let shard = shard.with_context(|_| ErrorKind::PrimaryStoreRead("shard"))?;
            let event = Event::builder().snapshot().shard(shard);
            let code = event.code();
            let stream_key = event.stream_key();
            let event = EmitMessage::with(stream_key, event)
                .with_context(|_| ErrorKind::EventEmit(code))?
                .trace(span.context().clone());
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::EventEmit(code))?;
        }
        Ok(())
    }
}

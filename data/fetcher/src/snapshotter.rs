use failure::ResultExt;

use replicante_data_models::Event;
use replicante_data_store::Store;
use replicante_streams_events::EventsStream;

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

    pub fn run(&self) -> Result<()> {
        self.discovery()?;
        self.agents()?;
        self.nodes()?;
        self.shards()?;
        Ok(())
    }
}

impl Snapshotter {
    fn agents(&self) -> Result<()> {
        let statuses = self.store.cluster_agents(self.cluster.clone())
            .with_context(|_| ErrorKind::StoreRead("agents statuses"))?;
        for status in statuses {
            let status = status.with_context(|_| ErrorKind::StoreRead("agent status"))?;
            let event = Event::builder().snapshot().agent(status);
            let code = event.code();
            self.events.emit(event).with_context(|_| ErrorKind::EventEmit(code))?;
        }
        let infos = self.store.cluster_agents_info(self.cluster.clone())
            .with_context(|_| ErrorKind::StoreRead("agents info"))?;
        for info in infos {
            let info = info.with_context(|_| ErrorKind::StoreRead("agent info"))?;
            let event = Event::builder().snapshot().agent_info(info);
            let code = event.code();
            self.events.emit(event).with_context(|_| ErrorKind::EventEmit(code))?;
        }
        Ok(())
    }

    fn discovery(&self) -> Result<()> {
        let discovery = self.store.cluster_discovery(self.cluster.clone())
            .with_context(|_| ErrorKind::StoreRead("discovery"))?;
        if let Some(discovery) = discovery {
            let event = Event::builder().snapshot().discovery(discovery);
            let code = event.code();
            self.events.emit(event).with_context(|_| ErrorKind::EventEmit(code))?;
        }
        Ok(())
    }

    fn nodes(&self) -> Result<()> {
        let nodes = self.store.cluster_nodes(self.cluster.clone())
            .with_context(|_| ErrorKind::StoreRead("nodes"))?;
        for node in nodes {
            let node = node.with_context(|_| ErrorKind::StoreRead("node"))?;
            let event = Event::builder().snapshot().node(node);
            let code = event.code();
            self.events.emit(event).with_context(|_| ErrorKind::EventEmit(code))?;
        }
        Ok(())
    }

    fn shards(&self) -> Result<()> {
        let shards = self.store.cluster_shards(self.cluster.clone())
            .with_context(|_| ErrorKind::StoreRead("shards"))?;
        for shard in shards {
            let shard = shard.with_context(|_| ErrorKind::StoreRead("shard"))?;
            let event = Event::builder().snapshot().shard(shard);
            let code = event.code();
            self.events.emit(event).with_context(|_| ErrorKind::EventEmit(code))?;
        }
        Ok(())
    }
}

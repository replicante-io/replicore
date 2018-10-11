use replicante_data_models::Event;
use replicante_data_store::Store;
use replicante_streams_events::EventsStream;

use super::Result;
use super::ResultExt;


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
        let statuses = self.store.cluster_agents(self.cluster.clone())?;
        for status in statuses {
            let status = status?;
            let event = Event::builder().snapshot().agent(status);
            self.events.emit(event).chain_err(|| "Error emitting agent snapshot")?;
        }
        let infos = self.store.cluster_agents_info(self.cluster.clone())?;
        for info in infos {
            let info = info?;
            let event = Event::builder().snapshot().agent_info(info);
            self.events.emit(event).chain_err(|| "Error emitting agent info snapshot")?;
        }
        Ok(())
    }

    fn discovery(&self) -> Result<()> {
        let discovery = self.store.cluster_discovery(self.cluster.clone())?;
        if let Some(discovery) = discovery {
            let event = Event::builder().snapshot().discovery(discovery);
            self.events.emit(event).chain_err(|| "Error emitting discovery snapshot")?;
        }
        Ok(())
    }

    fn nodes(&self) -> Result<()> {
        let nodes = self.store.cluster_nodes(self.cluster.clone())?;
        for node in nodes {
            let node = node?;
            let event = Event::builder().snapshot().node(node);
            self.events.emit(event).chain_err(|| "Error emitting node snapshot")?;
        }
        Ok(())
    }

    fn shards(&self) -> Result<()> {
        let shards = self.store.cluster_shards(self.cluster.clone())?;
        for shard in shards {
            let shard = shard?;
            let event = Event::builder().snapshot().shard(shard);
            self.events.emit(event).chain_err(|| "Error emitting shard snapshot")?;
        }
        Ok(())
    }
}

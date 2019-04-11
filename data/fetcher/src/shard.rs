use failure::ResultExt;

use replicante_agent_client::Client;
use replicante_data_models::Event;
use replicante_data_models::Shard;

use replicante_data_store::Store;
use replicante_streams_events::EventsStream;

use super::Error;
use super::ErrorKind;
use super::Result;


/// Subset of fetcher logic that deals specifically with shards.
pub struct ShardFetcher {
    events: EventsStream,
    store: Store,
}

impl ShardFetcher {
    pub fn new(events: EventsStream, store: Store) -> ShardFetcher {
        ShardFetcher {
            events,
            store,
        }
    }

    pub fn process_shards(&self, client: &Client, cluster: &str, node: &str) -> Result<()> {
        let shards = client.shards()
            .with_context(|_| ErrorKind::AgentRead("shards", client.id().to_string()))?;
        for shard in shards.shards {
            let shard = Shard::new(cluster.to_string(), node.to_string(), shard);
            // TODO(stefano): should an error prevent all following shards from being processed?
            self.process_shard(shard)?;
        }
        Ok(())
    }
}

impl ShardFetcher {
    fn process_shard(&self, shard: Shard) -> Result<()> {
        let cluster_id = shard.cluster_id.clone();
        let node = shard.node.clone();
        let id = shard.id.clone();
        match self.store.shard(cluster_id.clone(), node.clone(), id.clone()) {
            Err(error) => Err(error)
                .with_context(|_| ErrorKind::StoreRead("shard"))
                .map_err(Error::from),
            Ok(None) => self.process_shard_new(shard),
            Ok(Some(old)) => self.process_shard_existing(shard, old)
        }
    }

    fn process_shard_existing(&self, shard: Shard, old: Shard) -> Result<()> {
        // If the shard is the same (including offset and lag) exit now.
        if shard == old {
            return Ok(());
        }

        // If anything other then offset or lag changed emit and event.
        if self.shard_changed(&shard, &old) {
            let event = Event::builder().shard().allocation_changed(old, shard.clone());
            let code = event.code();
            self.events.emit(event).with_context(|_| ErrorKind::EventEmit(code))?;
        }

        // Persist the model so the latest offset and lag information are available.
        self.store.persist_shard(shard)
            .with_context(|_| ErrorKind::StoreWrite("shard update")).map_err(Error::from)
    }

    fn process_shard_new(&self, shard: Shard) -> Result<()> {
        let event = Event::builder().shard().shard_allocation_new(shard.clone());
        let code = event.code();
        self.events.emit(event).with_context(|_| ErrorKind::EventEmit(code))?;
        self.store.persist_shard(shard)
            .with_context(|_| ErrorKind::StoreWrite("new shard")).map_err(Error::from)
    }

    /// Checks if a shard has changed since the last fetch.
    ///
    /// Because shard data includes commit offsets and lag we need to do a more
    /// in-depth comparison to ignore "expected" changes.
    fn shard_changed(&self, shard: &Shard, old: &Shard) -> bool {
        // Easy case: they are the same.
        if shard == old {
            return false;
        }
        // Check if the "stable" attributes have changed.
        shard.cluster_id != old.cluster_id ||
            shard.id != old.id ||
            shard.node != old.node ||
            shard.role != old.role
    }
}

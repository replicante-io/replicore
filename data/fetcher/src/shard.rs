use replicante_agent_client::Client;
use replicante_data_models::Event;
use replicante_data_models::Shard;
use replicante_data_store::Store;

use super::Result;
use super::ResultExt;


const FAIL_FIND_SHARD: &str = "Failed to fetch shard";
const FAIL_PERSIST_SHARD: &str = "Failed to persist shard";


/// Subset of fetcher logic that deals specifically with shards.
pub struct ShardFetcher {
    store: Store,
}

impl ShardFetcher {
    pub fn new(store: Store) -> ShardFetcher {
        ShardFetcher {
            store,
        }
    }

    pub fn process_shards(&self, client: &Client, cluster: &str, node: &str) -> Result<()> {
        let shards = client.shards()?;
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
        let cluster = shard.cluster.clone();
        let node = shard.node.clone();
        let id = shard.id.clone();
        match self.store.shard(cluster.clone(), node.clone(), id.clone()) {
            Err(error) => Err(error).chain_err(|| FAIL_FIND_SHARD),
            Ok(None) => self.process_shard_new(shard),
            Ok(Some(old)) => self.process_shard_existing(shard, old)
        }
    }

    fn process_shard_existing(&self, shard: Shard, old: Shard) -> Result<()> {
        if shard == old {
            return Ok(());
        }
        // TODO(stefano): emit shard changed events.
        self.store.persist_shard(shard).chain_err(|| FAIL_PERSIST_SHARD)
    }

    fn process_shard_new(&self, shard: Shard) -> Result<()> {
        let event = Event::builder().shard().shard_allocation_new(shard.clone());
        self.store.persist_event(event).chain_err(|| FAIL_PERSIST_SHARD)?;
        self.store.persist_shard(shard).chain_err(|| FAIL_PERSIST_SHARD)
    }
}

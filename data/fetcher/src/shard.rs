use error_chain::ChainedError;
use slog::Logger;

use replicante_agent_client::Client;
use replicante_data_models::Shard;
use replicante_data_store::Store;

use super::Result;
use super::metrics::FETCHER_ERRORS_COUNT;


/// Subset of fetcher logic that deals specifically with shards.
pub struct ShardFetcher {
    logger: Logger,
    store: Store,
}

impl ShardFetcher {
    pub fn new(logger: Logger, store: Store) -> ShardFetcher {
        ShardFetcher {
            logger,
            store,
        }
    }

    pub fn persist_shard(&self, shard: Shard) {
        let cluster = shard.cluster.clone();
        let node = shard.node.clone();
        let id = shard.id.clone();
        let old = match self.store.shard(cluster.clone(), node.clone(), id.clone()) {
            Ok(old) => old,
            Err(error) => {
                FETCHER_ERRORS_COUNT.with_label_values(&[&cluster]).inc();
                let error = error.display_chain().to_string();
                error!(
                    self.logger, "Failed to fetch shard info";
                    "cluster" => cluster, "node" => node, "id" => id,
                    "error" => error
                );
                return;
            }
        };

        // TODO: Emit shard events.

        if old != Some(shard.clone()) {
            match self.store.persist_shard(shard) {
                Ok(_) => (),
                Err(error) => {
                    FETCHER_ERRORS_COUNT.with_label_values(&[&cluster]).inc();
                    let error = error.display_chain().to_string();
                    error!(
                        self.logger, "Failed to persist node info";
                        "cluster" => cluster, "node" => node, "id" => id,
                        "error" => error
                    );
                }
            };
        }
    }

    pub fn process_shards(&self, client: &Client, cluster: String, node: String) -> Result<()> {
        let status = client.status()?;
        for shard in status.shards {
            let shard = Shard::new(cluster.clone(), node.clone(), shard);
            self.persist_shard(shard);
        }
        Ok(())
    }
}

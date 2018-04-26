use error_chain::ChainedError;
use slog::Logger;

use replicante_agent_client::Client;
use replicante_data_models::Node;
use replicante_data_store::Store;

use super::Result;
use super::meta::ClusterMetaBuilder;
use super::metrics::FETCHER_ERRORS_COUNT;


/// Subset of fetcher logic that deals specifically with nodes.
pub struct NodeFetcher {
    logger: Logger,
    store: Store,
}

impl NodeFetcher {
    pub fn new(logger: Logger, store: Store) -> NodeFetcher {
        NodeFetcher {
            logger,
            store,
        }
    }

    pub fn persist_node(&self, node: Node) {
        let cluster = node.cluster.clone();
        let name = node.name.clone();
        let old = match self.store.node(cluster.clone(), name.clone()) {
            Ok(old) => old,
            Err(error) => {
                FETCHER_ERRORS_COUNT.with_label_values(&[&name]).inc();
                let error = error.display_chain().to_string();
                error!(
                    self.logger, "Failed to fetch node info";
                    "cluster" => cluster, "name" => name, "error" => error
                );
                return;
            }
        };

        // TODO: Emit node events.

        if old != Some(node.clone()) {
            match self.store.persist_node(node) {
                Ok(_) => (),
                Err(error) => {
                    FETCHER_ERRORS_COUNT.with_label_values(&[&name]).inc();
                    let error = error.display_chain().to_string();
                    error!(
                        self.logger, "Failed to persist node info";
                        "cluster" => cluster, "name" => name, "error" => error
                    );
                }
            };
        }
    }

    pub fn process_node(&self, client: &Client, meta: &mut ClusterMetaBuilder) -> Result<()> {
        let info = client.info()?;
        let node = Node::new(info.datastore);
        meta.node_kind(node.kind.clone());
        self.persist_node(node);
        Ok(())
    }
}

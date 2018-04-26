use std::collections::HashSet;

use error_chain::ChainedError;
use slog::Logger;

use replicante_data_models::ClusterMeta;
use replicante_data_store::Store;

use super::metrics::FETCHER_ERRORS_COUNT;


pub struct ClusterMetaBuilder {
    cluster: String,
    kinds: HashSet<String>,
    nodes: i32,
}

impl ClusterMetaBuilder {
    pub fn build(self) -> ClusterMeta {
        let mut meta = ClusterMeta::new(self.cluster, "<OVERRIDE>", self.nodes);
        meta.kinds = self.kinds.into_iter().collect();
        meta
    }

    pub fn new(cluster: String) -> ClusterMetaBuilder {
        ClusterMetaBuilder {
            cluster,
            kinds: HashSet::new(),
            nodes: 0,
        }
    }

    pub fn node_inc(&mut self) {
        self.nodes += 1;
    }

    pub fn node_kind(&mut self, kind: String) {
        self.kinds.insert(kind);
    }
}


/// Subset of fetcher logic that deals specifically with cluster metadata.
pub struct MetaFetcher {
    logger: Logger,
    store: Store,
}

impl MetaFetcher {
    pub fn new(logger: Logger, store: Store) -> MetaFetcher {
        MetaFetcher {
            logger,
            store,
        }
    }

    pub fn persist_meta(&self, meta: ClusterMeta) {
        let name = meta.name.clone();
        match self.store.persist_cluster_meta(meta) {
            Ok(_) => (),
            Err(error) => {
                FETCHER_ERRORS_COUNT.with_label_values(&[&name]).inc();
                let error = error.display_chain().to_string();
                error!(
                    self.logger, "Failed to persist cluster metadata";
                    "cluster" => name, "error" => error
                );
            }
        };
    }
}

use std::collections::HashSet;

use replicante_data_models::ClusterMeta;
use replicante_data_store::Store;

use super::Result;
use super::ResultExt;


const FAIL_PERSIST_META: &str = "Failed to persist agent meta";


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
    store: Store,
}

impl MetaFetcher {
    pub fn new(store: Store) -> MetaFetcher {
        MetaFetcher {
            store,
        }
    }

    pub fn persist_meta(&self, meta: ClusterMeta) -> Result<()> {
        self.store.persist_cluster_meta(meta).chain_err(|| FAIL_PERSIST_META)
    }
}

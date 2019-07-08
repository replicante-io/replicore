use std::sync::Arc;
use std::sync::Mutex;

use opentracingrust::SpanContext;

use replicante_models_core::ClusterMeta;

use super::MockState;
use crate::backend::AgentImpl;
use crate::backend::AgentsImpl;
use crate::backend::ClusterImpl;
use crate::backend::LegacyImpl;
use crate::backend::LegacyInterface;
use crate::backend::NodeImpl;
use crate::backend::NodesImpl;
use crate::backend::PersistImpl;
use crate::backend::ShardImpl;
use crate::backend::ShardsImpl;
use crate::backend::StoreImpl;
use crate::backend::StoreInterface;
use crate::store::Store;
use crate::Cursor;
use crate::Result;

/// Mock implementation of the `StoreInterface`.
pub struct StoreMock {
    pub state: Arc<Mutex<MockState>>,
}

impl StoreInterface for StoreMock {
    fn agent(&self) -> AgentImpl {
        panic!("TODO: StoreMock::agent");
    }

    fn agents(&self) -> AgentsImpl {
        panic!("TODO: StoreMock::agents");
    }

    fn cluster(&self) -> ClusterImpl {
        panic!("TODO: StoreMock::cluster");
    }

    fn legacy(&self) -> LegacyImpl {
        let legacy = Legacy {
            state: Arc::clone(&self.state),
        };
        LegacyImpl::new(legacy)
    }

    fn node(&self) -> NodeImpl {
        panic!("TODO: StoreMock::node");
    }

    fn nodes(&self) -> NodesImpl {
        panic!("TODO: StoreMock::nodes");
    }

    fn persist(&self) -> PersistImpl {
        panic!("TODO: StoreMock::persist");
    }

    fn shard(&self) -> ShardImpl {
        panic!("TODO: StoreMock::shard");
    }

    fn shards(&self) -> ShardsImpl {
        panic!("TODO: StoreMock::shards");
    }
}

impl From<StoreMock> for Store {
    fn from(store: StoreMock) -> Store {
        let store = StoreImpl::new(store);
        Store::with_impl(store)
    }
}

/// Mock implementation of the `LegacyInterface`.
struct Legacy {
    state: Arc<Mutex<MockState>>,
}

impl LegacyInterface for Legacy {
    fn cluster_meta(
        &self,
        _cluster_id: String,
        _: Option<SpanContext>,
    ) -> Result<Option<ClusterMeta>> {
        panic!("mocking primary store::legacy::cluster_meta not yet supportd");
    }

    fn find_clusters(
        &self,
        _search: String,
        _limit: u8,
        _: Option<SpanContext>,
    ) -> Result<Cursor<ClusterMeta>> {
        panic!("mocking primary store::legacy::find_clusters not yet supportd");
    }

    fn persist_cluster_meta(&self, _meta: ClusterMeta, _: Option<SpanContext>) -> Result<()> {
        panic!("mocking primary store::legacy::persist_cluster_meta not yet supportd");
    }

    fn top_clusters(&self, _: Option<SpanContext>) -> Result<Cursor<ClusterMeta>> {
        let clusters = &self.state.lock().unwrap().clusters_meta;
        let mut results: Vec<ClusterMeta> = clusters.iter().map(|(_, meta)| meta.clone()).collect();
        results.sort_by_key(|meta| meta.nodes);
        results.reverse();
        let results: Vec<Result<ClusterMeta>> = results.into_iter().map(Ok).collect();
        let cursor = Cursor(Box::new(results.into_iter()));
        Ok(cursor)
    }
}

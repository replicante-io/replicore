use std::sync::Arc;
use std::sync::Mutex;

use replicante_data_models::ClusterMeta;
use replicante_data_models::Event;

use super::super::backend::AgentImpl;
use super::super::backend::AgentsImpl;
use super::super::backend::ClusterImpl;
use super::super::backend::LegacyImpl;
use super::super::backend::LegacyInterface;
use super::super::backend::NodeImpl;
use super::super::backend::NodesImpl;
use super::super::backend::PersistImpl;
use super::super::backend::ShardImpl;
use super::super::backend::ShardsImpl;
use super::super::backend::StoreImpl;
use super::super::backend::StoreInterface;
use super::super::store::legacy::EventsFilters;
use super::super::store::legacy::EventsOptions;
use super::super::store::Store;
use super::super::Cursor;
use super::super::Result;
use super::MockState;

/// Mock implementation of the `StoreInterface`.
pub struct StoreMock {
    pub state: Arc<Mutex<MockState>>,
}

impl StoreInterface for StoreMock {
    fn agent(&self) -> AgentImpl {
        panic!("TODO");
    }

    fn agents(&self) -> AgentsImpl {
        panic!("TODO");
    }

    fn cluster(&self) -> ClusterImpl {
        panic!("TODO");
    }

    fn legacy(&self) -> LegacyImpl {
        let legacy = Legacy {
            state: Arc::clone(&self.state),
        };
        LegacyImpl::new(legacy)
    }

    fn node(&self) -> NodeImpl {
        panic!("TODO");
    }

    fn nodes(&self) -> NodesImpl {
        panic!("TODO");
    }

    fn persist(&self) -> PersistImpl {
        panic!("TODO");
    }

    fn shard(&self) -> ShardImpl {
        panic!("TODO");
    }

    fn shards(&self) -> ShardsImpl {
        panic!("TODO");
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
    fn cluster_meta(&self, _cluster_id: String) -> Result<Option<ClusterMeta>> {
        panic!("mocking primary store::legacy::cluster_meta not yet supportd");
    }

    fn events(&self, _filters: EventsFilters, _options: EventsOptions) -> Result<Cursor<Event>> {
        panic!("mocking primary store::legacy::events not yet supportd");
    }

    fn find_clusters(&self, _search: String, _limit: u8) -> Result<Cursor<ClusterMeta>> {
        panic!("mocking primary store::legacy::find_clusters not yet supportd");
    }

    fn persist_cluster_meta(&self, _meta: ClusterMeta) -> Result<()> {
        panic!("mocking primary store::legacy::persist_cluster_meta not yet supportd");
    }

    fn persist_event(&self, _event: Event) -> Result<()> {
        panic!("mocking primary store::legacy::persist_event not yet supportd");
    }

    fn top_clusters(&self) -> Result<Cursor<ClusterMeta>> {
        let clusters = &self.state.lock().unwrap().clusters_meta;
        let mut results: Vec<ClusterMeta> = clusters.iter().map(|(_, meta)| meta.clone()).collect();
        results.sort_by_key(|meta| meta.nodes);
        results.reverse();
        let results: Vec<Result<ClusterMeta>> = results.into_iter().map(Ok).collect();
        let cursor = Cursor(Box::new(results.into_iter()));
        Ok(cursor)
    }
}

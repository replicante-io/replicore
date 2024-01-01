//! Inefficient in-memory implementation of [`Store`](super::Store) for unit tests.
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use anyhow::Result;
use futures::StreamExt;

use replisdk::core::models::api::NamespaceEntry;
use replisdk::core::models::cluster::ClusterSpec;
use replisdk::core::models::namespace::Namespace;
use replisdk::core::models::platform::Platform;

use replicore_context::Context;

use super::DeleteOps;
use super::DeleteResponses;
use super::PersistOps;
use super::PersistResponses;
use super::QueryOps;
use super::QueryResponses;
use super::StoreBackend;

/// In-memory implementation of a mock [`Store`](super::Store) for unit tests.
#[derive(Clone)]
pub struct StoreFixture {
    /// Shared in-memory state to mock the DB with.
    inner: Arc<Mutex<StoreFixtureState>>,
}

impl StoreFixture {
    /// Lock and access the shared inner store.
    fn access(&self) -> MutexGuard<StoreFixtureState> {
        self.inner
            .lock()
            .expect("StoreFixture::inner state lock poisoned")
    }
}

impl Default for StoreFixture {
    fn default() -> Self {
        let inner = StoreFixtureState::default();
        let inner = Mutex::new(inner);
        let inner = Arc::new(inner);
        StoreFixture { inner }
    }
}

#[async_trait::async_trait]
impl StoreBackend for StoreFixture {
    async fn delete(&self, _: &Context, op: DeleteOps) -> Result<DeleteResponses> {
        let mut store = self.access();
        match op {
            DeleteOps::ClusterSpec(cluster) => {
                let key = (cluster.0.ns_id, cluster.0.name);
                store.cluster_specs.remove(&key);
            }
            DeleteOps::Namespace(ns) => {
                store.namespaces.remove(&ns.0.id);
            }
            DeleteOps::Platform(pl) => {
                let key = (pl.0.ns_id, pl.0.name);
                store.platforms.remove(&key);
            }
        };
        Ok(DeleteResponses::Success)
    }

    async fn query(&self, _: &Context, op: QueryOps) -> Result<QueryResponses> {
        let store = self.access();
        match op {
            QueryOps::ClusterSpec(cluster) => {
                let key = (cluster.ns_id, cluster.name);
                let spec = store.cluster_specs.get(&key).cloned();
                Ok(QueryResponses::ClusterSpec(spec))
            }
            QueryOps::ListClusterSpecIds(query) => {
                let mut ids = Vec::new();
                for (ns, id) in store.cluster_specs.keys() {
                    if ns == query.id.as_str() {
                        ids.push(id.to_string());
                    }
                }
                let ids = futures::stream::iter(ids).map(Ok).boxed();
                Ok(QueryResponses::StringStream(ids))
            }
            QueryOps::ListNamespaces => {
                let items: Vec<_> = store
                    .namespaces
                    .iter()
                    .map(|(_, ns)| {
                        let id = ns.id.clone();
                        let status = ns.status.clone();
                        NamespaceEntry { id, status }
                    })
                    .collect();
                let items = futures::stream::iter(items).map(Ok).boxed();
                Ok(QueryResponses::NamespaceEntries(items))
            }
            QueryOps::ListPlatformIds(query) => {
                let mut ids = Vec::new();
                for (ns, id) in store.platforms.keys() {
                    if ns == query.id.as_str() {
                        ids.push(id.to_string());
                    }
                }
                let ids = futures::stream::iter(ids).map(Ok).boxed();
                Ok(QueryResponses::StringStream(ids))
            }
            QueryOps::Namespace(ns) => {
                let ns = store.namespaces.get(&ns.0.id).cloned();
                Ok(QueryResponses::Namespace(ns))
            }
            QueryOps::Platform(query) => {
                let key = (query.ns_id, query.name);
                let platform = store.platforms.get(&key).cloned();
                Ok(QueryResponses::Platform(platform))
            }
        }
    }

    async fn persist(&self, _: &Context, op: PersistOps) -> Result<PersistResponses> {
        let mut store = self.access();
        match op {
            PersistOps::ClusterSpec(spec) => {
                let key = (spec.ns_id.clone(), spec.cluster_id.clone());
                store.cluster_specs.insert(key, spec);
            }
            PersistOps::Namespace(ns) => {
                store.namespaces.insert(ns.id.clone(), ns);
            }
            PersistOps::Platform(platform) => {
                let key = (platform.ns_id.clone(), platform.name.clone());
                store.platforms.insert(key, platform);
            }
        };
        Ok(PersistResponses::Success)
    }
}

/// Container for the shared state.
#[derive(Default)]
struct StoreFixtureState {
    cluster_specs: HashMap<(String, String), ClusterSpec>,
    namespaces: HashMap<String, Namespace>,
    platforms: HashMap<(String, String), Platform>,
}

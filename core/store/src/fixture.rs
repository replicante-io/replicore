//! Inefficient in-memory implementation of [`Store`](super::Store) for unit tests.
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use anyhow::Result;
use futures::StreamExt;
use uuid::Uuid;

use replisdk::core::models::api::ClusterSpecEntry;
use replisdk::core::models::api::NamespaceEntry;
use replisdk::core::models::api::OActionEntry;
use replisdk::core::models::api::PlatformEntry;
use replisdk::core::models::cluster::ClusterDiscovery;
use replisdk::core::models::cluster::ClusterSpec;
use replisdk::core::models::namespace::Namespace;
use replisdk::core::models::oaction::OAction;
use replisdk::core::models::platform::Platform;

use replicore_cluster_models::ConvergeState;
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
            DeleteOps::ClusterConvergeState(cluster) => {
                let key = (cluster.0.ns_id, cluster.0.name);
                store.cluster_converge_states.remove(&key);
            }
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
            QueryOps::ClusterConvergeState(cluster) => {
                let key = (cluster.ns_id, cluster.name);
                let state = store.cluster_converge_states.get(&key).cloned();
                Ok(QueryResponses::ClusterConvergeState(state))
            }
            QueryOps::ClusterDiscovery(cluster) => {
                let key = (cluster.ns_id, cluster.name);
                let discovery = store.cluster_discoveries.get(&key).cloned();
                Ok(QueryResponses::ClusterDiscovery(discovery))
            }
            QueryOps::ClusterSpec(cluster) => {
                let key = (cluster.ns_id, cluster.name);
                let spec = store.cluster_specs.get(&key).cloned();
                Ok(QueryResponses::ClusterSpec(spec))
            }
            QueryOps::ListClusterSpecs(query) => {
                let mut items = Vec::new();
                for ((ns, _), spec) in store.cluster_specs.iter() {
                    if ns != query.id.as_str() {
                        continue;
                    }
                    let item = ClusterSpecEntry {
                        ns_id: ns.clone(),
                        cluster_id: spec.cluster_id.clone(),
                        active: spec.active,
                    };
                    items.push(item);
                }
                let items = futures::stream::iter(items).map(Ok).boxed();
                Ok(QueryResponses::ClusterSpecEntries(items))
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
            QueryOps::ListOActions(query) => {
                let mut items = Vec::new();
                for (_, action) in store.oactions.iter() {
                    if query.ns_id != action.ns_id {
                        continue;
                    }
                    if query.cluster_id != action.cluster_id {
                        continue;
                    }
                    if action.state.is_final() && !query.include_finished {
                        continue;
                    }
                    let item = OActionEntry {
                        ns_id: action.ns_id.clone(),
                        cluster_id: action.cluster_id.clone(),
                        action_id: action.action_id,
                        created_ts: action.created_ts,
                        finished_ts: action.finished_ts,
                        kind: action.kind.clone(),
                        state: action.state,
                    };
                    items.push(item);
                }
                let items = futures::stream::iter(items).map(Ok).boxed();
                Ok(QueryResponses::OActionEntries(items))
            }
            QueryOps::ListPlatforms(query) => {
                let mut items = Vec::new();
                for ((ns, _), platform) in store.platforms.iter() {
                    if ns != query.id.as_str() {
                        continue;
                    }
                    let item = PlatformEntry {
                        active: platform.active,
                        name: platform.name.clone(),
                    };
                    items.push(item);
                }
                let items = futures::stream::iter(items).map(Ok).boxed();
                Ok(QueryResponses::PlatformEntries(items))
            }
            QueryOps::Namespace(ns) => {
                let ns = store.namespaces.get(&ns.0.id).cloned();
                Ok(QueryResponses::Namespace(ns))
            }
            QueryOps::OAction(query) => {
                let key = (query.0.ns_id, query.0.cluster_id, query.0.action_id);
                let oaction = store.oactions.get(&key).cloned();
                Ok(QueryResponses::OAction(oaction))
            }
            QueryOps::Platform(query) => {
                let key = (query.ns_id, query.name);
                let platform = store.platforms.get(&key).cloned();
                Ok(QueryResponses::Platform(platform))
            }
            QueryOps::UnfinishedOAction(cluster) => {
                let actions: Vec<_> = store
                    .oactions
                    .iter()
                    .filter(|(_, action)| {
                        action.ns_id == cluster.ns_id
                            && action.cluster_id == cluster.name
                            && !action.state.is_final()
                    })
                    .map(|(_, action)| action.clone())
                    .collect();
                let actions = futures::stream::iter(actions).map(Ok).boxed();
                Ok(QueryResponses::OActions(actions))
            }
        }
    }

    async fn persist(&self, _: &Context, op: PersistOps) -> Result<PersistResponses> {
        let mut store = self.access();
        match op {
            PersistOps::ClusterConvergeState(state) => {
                let key = (state.ns_id.clone(), state.cluster_id.clone());
                store.cluster_converge_states.insert(key, state);
            }
            PersistOps::ClusterDiscovery(disc) => {
                let key = (disc.ns_id.clone(), disc.cluster_id.clone());
                store.cluster_discoveries.insert(key, disc);
            }
            PersistOps::ClusterSpec(spec) => {
                let key = (spec.ns_id.clone(), spec.cluster_id.clone());
                store.cluster_specs.insert(key, spec);
            }
            PersistOps::Namespace(ns) => {
                store.namespaces.insert(ns.id.clone(), ns);
            }
            PersistOps::OAction(oaction) => {
                let key = (
                    oaction.ns_id.clone(),
                    oaction.cluster_id.clone(),
                    oaction.action_id,
                );
                store.oactions.insert(key, oaction);
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
    cluster_converge_states: HashMap<(String, String), ConvergeState>,
    cluster_discoveries: HashMap<(String, String), ClusterDiscovery>,
    cluster_specs: HashMap<(String, String), ClusterSpec>,
    namespaces: HashMap<String, Namespace>,
    oactions: HashMap<(String, String, Uuid), OAction>,
    platforms: HashMap<(String, String), Platform>,
}

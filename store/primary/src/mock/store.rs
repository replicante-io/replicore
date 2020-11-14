use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use chrono::DateTime;
use chrono::Utc;
use opentracingrust::SpanContext;
use uuid::Uuid;

use replicante_models_core::actions::Action;
use replicante_models_core::actions::ActionState;
use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::Node;
use replicante_models_core::agent::Shard;
use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::discovery::DiscoverySettings;
use replicante_models_core::cluster::ClusterMeta;

use super::MockState;
use crate::backend::ActionsImpl;
use crate::backend::ActionsInterface;
use crate::backend::AgentImpl;
use crate::backend::AgentsImpl;
use crate::backend::ClusterImpl;
use crate::backend::GlobalSearchImpl;
use crate::backend::LegacyImpl;
use crate::backend::LegacyInterface;
use crate::backend::NodeImpl;
use crate::backend::NodesImpl;
use crate::backend::PersistImpl;
use crate::backend::PersistInterface;
use crate::backend::ShardImpl;
use crate::backend::ShardsImpl;
use crate::backend::StoreImpl;
use crate::backend::StoreInterface;
use crate::store::actions::ActionSyncState;
use crate::store::actions::ActionsAttributes;
use crate::store::Store;
use crate::Cursor;
use crate::Result;

/// Mock implementation of the `StoreInterface`.
pub struct StoreMock {
    pub state: Arc<Mutex<MockState>>,
}

impl StoreInterface for StoreMock {
    fn actions(&self) -> ActionsImpl {
        let actions = Actions {
            state: Arc::clone(&self.state),
        };
        ActionsImpl::new(actions)
    }

    fn agent(&self) -> AgentImpl {
        panic!("TODO: StoreMock::agent");
    }

    fn agents(&self) -> AgentsImpl {
        panic!("TODO: StoreMock::agents");
    }

    fn cluster(&self) -> ClusterImpl {
        panic!("TODO: StoreMock::cluster");
    }

    fn global_search(&self) -> GlobalSearchImpl {
        panic!("TODO: StoreMock::global_search");
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
        let persist = Persist {
            state: Arc::clone(&self.state),
        };
        PersistImpl::new(persist)
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

/// Mock implementation of the `ActionsInterface`.
struct Actions {
    state: Arc<Mutex<MockState>>,
}

impl ActionsInterface for Actions {
    fn approve(
        &self,
        _attrs: &ActionsAttributes,
        _action_id: Uuid,
        _: Option<SpanContext>,
    ) -> Result<()> {
        panic!("TODO: MockStore::Actions::approve")
    }

    fn disapprove(
        &self,
        _attrs: &ActionsAttributes,
        _action_id: Uuid,
        _: Option<SpanContext>,
    ) -> Result<()> {
        panic!("TODO: MockStore::Actions::disapprove")
    }

    fn iter_lost(
        &self,
        attrs: &ActionsAttributes,
        node_id: String,
        refresh_id: i64,
        finished_ts: DateTime<Utc>,
        _: Option<SpanContext>,
    ) -> Result<Cursor<Action>> {
        let store = self.state.lock().expect("MockStore state lock is poisoned");
        let cluster_id = &attrs.cluster_id;
        let cursor: Vec<Action> = store
            .actions
            .iter()
            .filter(|(key, action)| {
                key.0 == *cluster_id && key.1 == *node_id && action.refresh_id != refresh_id
            })
            .map(|(_, action)| {
                // Simulate the changes that will be performed by `mark_lost` for clients.
                let mut action = action.clone();
                action.state = ActionState::Lost;
                action.finished_ts = Some(finished_ts);
                action
            })
            .collect();
        Ok(Cursor::new(cursor.into_iter().map(Ok)))
    }

    fn mark_lost(
        &self,
        attrs: &ActionsAttributes,
        node_id: String,
        refresh_id: i64,
        finished_ts: DateTime<Utc>,
        _: Option<SpanContext>,
    ) -> Result<()> {
        let cluster_id = &attrs.cluster_id;
        let mut store = self.state.lock().expect("MockStore state lock is poisoned");
        let actions = store.actions.iter_mut().filter(|(key, action)| {
            key.0 == *cluster_id && key.1 == *node_id && action.refresh_id != refresh_id
        });
        for (_, action) in actions {
            action.state = ActionState::Lost;
            action.finished_ts = Some(finished_ts);
        }
        Ok(())
    }

    fn pending_schedule(
        &self,
        attrs: &ActionsAttributes,
        node_id: String,
        _: Option<SpanContext>,
    ) -> Result<Cursor<Action>> {
        let store = self.state.lock().expect("MockStore state lock is poisoned");
        let cluster_id = &attrs.cluster_id;
        let cursor: Vec<Action> = store
            .actions
            .iter()
            .filter(|(key, action)| {
                key.0 == *cluster_id
                    && key.1 == *node_id
                    && action.state == ActionState::PendingSchedule
            })
            .map(|(_, action)| action.clone())
            .collect();
        Ok(Cursor::new(cursor.into_iter().map(Ok)))
    }

    fn state_for_sync(
        &self,
        attrs: &ActionsAttributes,
        node_id: String,
        action_ids: &[Uuid],
        _: Option<SpanContext>,
    ) -> Result<HashMap<Uuid, ActionSyncState>> {
        let mut results = HashMap::new();
        // Check ids in the state map.
        let store = self.state.lock().expect("MockStore state lock is poisoned");
        for id in action_ids {
            let state = store
                .actions
                .get(&(attrs.cluster_id.clone(), node_id.clone(), *id))
                .map(|action| {
                    if action.finished_ts.is_some() {
                        ActionSyncState::Finished
                    } else {
                        ActionSyncState::Found(action.clone())
                    }
                })
                .unwrap_or(ActionSyncState::NotFound);
            results.insert(*id, state);
        }

        // Add other ids as not found.
        for id in action_ids {
            results.entry(*id).or_insert(ActionSyncState::NotFound);
        }
        Ok(results)
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

/// Mock implementation of the `PersistInterface`.
struct Persist {
    state: Arc<Mutex<MockState>>,
}

impl PersistInterface for Persist {
    fn action(&self, action: Action, _: Option<SpanContext>) -> Result<()> {
        let key = (
            action.cluster_id.clone(),
            action.node_id.clone(),
            action.action_id,
        );
        self.state
            .lock()
            .expect("MockStore state lock poisoned")
            .actions
            .insert(key, action);
        Ok(())
    }

    fn agent(&self, _agent: Agent, _: Option<SpanContext>) -> Result<()> {
        panic!("TODO: MockStore::Persist::agent")
    }

    fn agent_info(&self, _agent: AgentInfo, _: Option<SpanContext>) -> Result<()> {
        panic!("TODO: MockStore::Persist::agent_info")
    }

    fn cluster_discovery(
        &self,
        _discovery: ClusterDiscovery,
        _: Option<SpanContext>,
    ) -> Result<()> {
        panic!("TODO: MockStore::Persist::cluster_discovery")
    }

    fn discovery_settings(
        &self,
        _settings: DiscoverySettings,
        _: Option<SpanContext>,
    ) -> Result<()> {
        panic!("TODO: MockStore::Persist::discovery_settings")
    }

    fn next_discovery_run(
        &self,
        _settings: DiscoverySettings,
        _: Option<SpanContext>,
    ) -> Result<()> {
        panic!("TODO: MockStore::Persist::next_discovery_run")
    }

    fn node(&self, _node: Node, _: Option<SpanContext>) -> Result<()> {
        panic!("TODO: MockStore::Persist::node")
    }

    fn shard(&self, _shard: Shard, _: Option<SpanContext>) -> Result<()> {
        panic!("TODO: MockStore::Persist::shard")
    }
}

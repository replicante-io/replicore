use std::sync::Arc;
use std::sync::Mutex;

use opentracingrust::SpanContext;
use uuid::Uuid;

use replicante_models_core::actions::node::Action;
use replicante_models_core::actions::node::ActionSyncSummary;
use replicante_models_core::actions::orchestrator::OrchestratorAction;
use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::Node;
use replicante_models_core::agent::Shard;
use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::discovery::DiscoverySettings;
use replicante_models_core::cluster::ClusterMeta;
use replicante_models_core::cluster::ClusterSettings;

use super::MockState;
use crate::backend::ActionImpl;
use crate::backend::ActionInterface;
use crate::backend::ActionsImpl;
use crate::backend::ActionsInterface;
use crate::backend::AgentImpl;
use crate::backend::AgentsImpl;
use crate::backend::ClusterImpl;
use crate::backend::DiscoverySettingsImpl;
use crate::backend::GlobalSearchImpl;
use crate::backend::LegacyImpl;
use crate::backend::LegacyInterface;
use crate::backend::NodeImpl;
use crate::backend::NodesImpl;
use crate::backend::OrchestratorActionsImpl;
use crate::backend::PersistImpl;
use crate::backend::PersistInterface;
use crate::backend::ShardImpl;
use crate::backend::ShardsImpl;
use crate::backend::StoreImpl;
use crate::backend::StoreInterface;
use crate::store::action::ActionAttributes;
use crate::store::actions::ActionsAttributes;
use crate::store::Store;
use crate::Cursor;
use crate::Result;

/// Mock implementation of the `StoreInterface`.
pub struct StoreMock {
    pub state: Arc<Mutex<MockState>>,
}

impl StoreInterface for StoreMock {
    fn action(&self) -> ActionImpl {
        let action = ActionMock {
            state: Arc::clone(&self.state),
        };
        ActionImpl::new(action)
    }

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

    fn discovery_settings(&self) -> DiscoverySettingsImpl {
        panic!("TODO: StoreMock::discovery_settings");
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

    fn orchestrator_actions(&self) -> OrchestratorActionsImpl {
        panic!("TODO: StoreMock::orchestrator_actions");
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

/// Mock implementation of the `ActionInterface`.
struct ActionMock {
    state: Arc<Mutex<MockState>>,
}

impl ActionInterface for ActionMock {
    fn get(&self, attrs: &ActionAttributes, _: Option<SpanContext>) -> Result<Option<Action>> {
        let store = self.state.lock().expect("MockStore state lock is poisoned");
        let action = store.actions.iter().find_map(|(key, action)| {
            if key.0 == attrs.cluster_id && key.2 == attrs.action_id {
                Some(action)
            } else {
                None
            }
        });
        Ok(action.cloned())
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

    #[allow(clippy::needless_collect)]
    fn unfinished_summaries(
        &self,
        attrs: &ActionsAttributes,
        _: Option<SpanContext>,
    ) -> Result<Cursor<ActionSyncSummary>> {
        let store = self.state.lock().expect("MockStore state lock is poisoned");
        let cluster_id = &attrs.cluster_id;
        let cursor: Vec<ActionSyncSummary> = store
            .actions
            .iter()
            .filter(|(key, action)| key.0 == *cluster_id && action.finished_ts.is_none())
            .map(|(_, action)| ActionSyncSummary::from(action))
            .collect();
        Ok(Cursor::new(cursor.into_iter().map(Ok)))
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

    #[allow(clippy::needless_collect)]
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

    fn agent(&self, agent: Agent, _: Option<SpanContext>) -> Result<()> {
        let key = (agent.cluster_id.clone(), agent.host.clone());
        self.state
            .lock()
            .expect("MockStore state lock poisoned")
            .agents
            .insert(key, agent);
        Ok(())
    }

    fn agent_info(&self, agent: AgentInfo, _: Option<SpanContext>) -> Result<()> {
        let key = (agent.cluster_id.clone(), agent.host.clone());
        self.state
            .lock()
            .expect("MockStore state lock poisoned")
            .agents_info
            .insert(key, agent);
        Ok(())
    }

    fn cluster_discovery(
        &self,
        _discovery: ClusterDiscovery,
        _: Option<SpanContext>,
    ) -> Result<()> {
        panic!("TODO: MockStore::Persist::cluster_discovery")
    }

    fn cluster_settings(&self, _settings: ClusterSettings, _: Option<SpanContext>) -> Result<()> {
        panic!("TODO: MockStore::Persist::cluster_settings")
    }

    fn discovery_settings(
        &self,
        _settings: DiscoverySettings,
        _: Option<SpanContext>,
    ) -> Result<()> {
        panic!("TODO: MockStore::Persist::discovery_settings")
    }

    fn next_cluster_orchestrate(
        &self,
        _settings: ClusterSettings,
        _: Option<SpanContext>,
    ) -> Result<()> {
        panic!("TODO: MockStore::Persist::next_cluster_orchestrate")
    }

    fn next_discovery_run(
        &self,
        _settings: DiscoverySettings,
        _: Option<SpanContext>,
    ) -> Result<()> {
        panic!("TODO: MockStore::Persist::next_discovery_run")
    }

    fn node(&self, node: Node, _: Option<SpanContext>) -> Result<()> {
        let key = (node.cluster_id.clone(), node.node_id.clone());
        self.state
            .lock()
            .expect("MockStore state lock poisoned")
            .nodes
            .insert(key, node);
        Ok(())
    }

    fn orchestrator_action(&self, _: OrchestratorAction, _: Option<SpanContext>) -> Result<()> {
        panic!("TODO: MockStore::Persist::orchestrator_action")
    }

    fn shard(&self, shard: Shard, _: Option<SpanContext>) -> Result<()> {
        let key = (
            shard.cluster_id.clone(),
            shard.node_id.clone(),
            shard.shard_id.clone(),
        );
        self.state
            .lock()
            .expect("MockStore state lock poisoned")
            .shards
            .insert(key, shard);
        Ok(())
    }
}

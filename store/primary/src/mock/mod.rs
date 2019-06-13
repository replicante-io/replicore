use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use replicante_models_core::Agent;
use replicante_models_core::AgentInfo;
use replicante_models_core::ClusterDiscovery;
use replicante_models_core::ClusterMeta;
use replicante_models_core::Event;
use replicante_models_core::Node;
use replicante_models_core::Shard;

use super::admin::Admin;
use super::store::Store;

mod store;

/// Manage a mocked store and admin interface.
#[derive(Clone, Default)]
pub struct Mock {
    pub state: Arc<Mutex<MockState>>,
}

impl Mock {
    /// Return an `Admin` "view" into the mock.
    pub fn admin(&self) -> Admin {
        panic!("mocking primary store admin interface not yet supportd");
    }

    /// Return a `Store` "view" into the mock.
    pub fn store(&self) -> Store {
        let store = self::store::StoreMock {
            state: Arc::clone(&self.state),
        };
        store.into()
    }
}

/// Internal mock state.
#[derive(Default)]
pub struct MockState {
    pub agents: HashMap<(String, String), Agent>,
    pub agents_info: HashMap<(String, String), AgentInfo>,
    pub clusters_meta: HashMap<String, ClusterMeta>,
    pub discoveries: HashMap<String, ClusterDiscovery>,
    pub events: Vec<Event>,
    pub nodes: HashMap<(String, String), Node>,
    pub shards: HashMap<(String, String, String), Shard>,
}

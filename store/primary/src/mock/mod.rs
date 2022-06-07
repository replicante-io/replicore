use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use uuid::Uuid;

use replicante_models_core::actions::node::Action;
use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::Node;
use replicante_models_core::agent::Shard;
use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::ClusterMeta;
use replicante_models_core::events::Event;

use crate::admin::Admin;
use crate::store::Store;

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
    pub actions: HashMap<(String, String, Uuid), Action>,
    pub agents: HashMap<(String, String), Agent>,
    pub agents_info: HashMap<(String, String), AgentInfo>,
    pub clusters_meta: HashMap<String, ClusterMeta>,
    pub discoveries: HashMap<String, ClusterDiscovery>,
    pub events: Vec<Event>,
    pub nodes: HashMap<(String, String), Node>,
    pub shards: HashMap<(String, String, String), Shard>,
}

//! Inefficient in-memory implementation of [`Store`](super::Store) for unit tests.
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use anyhow::Result;

use replisdk::core::models::namespace::Namespace;

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
            DeleteOps::Namespace(ns) => store.namespaces.remove(&ns.id),
        };
        Ok(DeleteResponses::Success)
    }

    async fn query(&self, _: &Context, op: QueryOps) -> Result<QueryResponses> {
        let store = self.access();
        match op {
            QueryOps::Namespace(ns) => {
                let ns = store.namespaces.get(&ns.id).cloned();
                Ok(QueryResponses::Namespace(ns))
            }
        }
    }

    async fn persist(&self, _: &Context, op: PersistOps) -> Result<PersistResponses> {
        let mut store = self.access();
        match op {
            PersistOps::Namespace(ns) => store.namespaces.insert(ns.id.clone(), ns),
        };
        Ok(PersistResponses::Success)
    }
}

/// Container for the shared state.
#[derive(Default)]
struct StoreFixtureState {
    namespaces: HashMap<String, Namespace>,
}

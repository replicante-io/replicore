use std::sync::Arc;

use opentracingrust::Tracer;
use slog::Logger;

use replicante_service_healthcheck::HealthChecks;

use crate::backend::backend_factory;
use crate::backend::StoreImpl;
use crate::Config;
use crate::Result;

pub mod actions;
pub mod cluster;
pub mod events;
pub mod orchestrator_actions;
pub mod persist;

use self::actions::Actions;
use self::cluster::Cluster;
use self::events::Events;
use self::orchestrator_actions::OrchestratorActions;
use self::persist::Persist;

/// Interface to Replicante view store layer.
///
/// This interface abstracts every interaction with the persistence layer and
/// hides implementation details about storage software and data encoding.
///
/// # Purpose
/// The view store is responsible for data used to respond to API requests
/// or to provide more context, debugging data, introspection, and similar data.
/// No data in the view store is used by Replicante Core to perform its function.
///
/// # Concurrency and transactions
/// The store does not provide a transactional interface.
/// Concurrency is allowed by sharding, with processes relying on the coordinator to avoid
/// stepping over each others toes.
///
/// The non-transactional, distributed, nature of a cluster state limits the value
/// of transactions when it comes to requirements around the cluster state.
#[derive(Clone)]
pub struct Store {
    store: StoreImpl,
}

impl Store {
    /// Instantiate a new store interface.
    pub fn new<T>(
        config: Config,
        logger: Logger,
        healthchecks: &mut HealthChecks,
        tracer: T,
    ) -> Result<Store>
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let store = backend_factory(config, logger, healthchecks, tracer)?;
        Ok(Store { store })
    }

    /// Instantiate a store with the given implementation.
    #[cfg(any(test, feature = "with_test_support"))]
    pub(crate) fn with_impl(store: StoreImpl) -> Store {
        Store { store }
    }

    /// Operate on actions.
    pub fn actions(&self, cluster_id: String) -> Actions {
        let actions = self.store.actions(cluster_id);
        Actions::new(actions)
    }

    /// Operate on a specific Cluster.
    pub fn cluster<'query>(
        &self,
        namespace: &'query str,
        cluster_id: &'query str,
    ) -> Cluster<'query> {
        let attrs = cluster::ClusterAttributes::new(namespace, cluster_id);
        let cluster = self.store.cluster();
        Cluster::new(cluster, attrs)
    }

    /// Operate on events.
    pub fn events(&self) -> Events {
        let events = self.store.events();
        Events::new(events)
    }

    /// Operate on orchestrator actions.
    pub fn orchestrator_actions<S>(&self, cluster_id: S) -> OrchestratorActions
    where
        S: Into<String>,
    {
        let actions = self.store.orchestrator_actions(cluster_id.into());
        OrchestratorActions::new(actions)
    }

    /// Persist (insert or update) models to the store.
    pub fn persist(&self) -> Persist {
        let persist = self.store.persist();
        Persist::new(persist)
    }
}

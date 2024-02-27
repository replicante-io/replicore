use chrono::DateTime;
use chrono::Utc;
use opentracingrust::SpanContext;
use uuid::Uuid;

use replicante_models_core::actions::node::Action;
use replicante_models_core::actions::node::ActionHistory;
use replicante_models_core::actions::orchestrator::OrchestratorAction;
use replicante_models_core::cluster::OrchestrateReport;
use replicante_models_core::events::Event;

use crate::backend::ActionsImpl;
use crate::backend::ActionsInterface;
use crate::backend::ClusterImpl;
use crate::backend::ClusterInterface;
use crate::backend::EventsImpl;
use crate::backend::OrchestratorActionsImpl;
use crate::backend::PersistImpl;
use crate::backend::PersistInterface;
use crate::backend::StoreImpl;
use crate::backend::StoreInterface;
use crate::store::actions::SearchFilters as ActionsSearchFilters;
use crate::store::cluster::ClusterAttributes;
use crate::store::Store;
use crate::Cursor;
use crate::Result;

/// Mock implementation of the `StoreInterface`.
pub struct StoreMock {
    // TODO: implement when needed.
}

impl StoreInterface for StoreMock {
    fn actions(&self, _: String) -> ActionsImpl {
        let actions = Actions {};
        ActionsImpl::new(actions)
    }

    fn cluster(&self) -> ClusterImpl {
        let cluster = Cluster {};
        ClusterImpl::new(cluster)
    }

    fn events(&self) -> EventsImpl {
        panic!("TODO: StoreMock::events")
    }

    fn orchestrator_actions(&self, _: String) -> OrchestratorActionsImpl {
        panic!("TODO: StoreMock::orchestrator_actions")
    }

    fn persist(&self) -> PersistImpl {
        let persist = Persist {};
        PersistImpl::new(persist)
    }
}

impl From<StoreMock> for Store {
    fn from(store: StoreMock) -> Store {
        let store = StoreImpl::new(store);
        Store::with_impl(store)
    }
}

struct Actions {
    // TODO: implement when needed.
}

impl ActionsInterface for Actions {
    fn action(&self, _: Uuid, _: Option<SpanContext>) -> Result<Option<Action>> {
        Ok(None)
    }

    fn finish_history(&self, _: Uuid, _: DateTime<Utc>, _: Option<SpanContext>) -> Result<()> {
        Ok(())
    }

    fn history(&self, _: Uuid, _: Option<SpanContext>) -> Result<Vec<ActionHistory>> {
        Ok(Vec::new())
    }

    fn search(&self, _: ActionsSearchFilters, _: Option<SpanContext>) -> Result<Cursor<Action>> {
        panic!("TODO: MockStore::actions::search")
    }
}

struct Cluster {
    // TODO: implement when needed.
}

impl ClusterInterface for Cluster {
    fn orchestrate_report(
        &self,
        _: &ClusterAttributes,
        _: Option<SpanContext>,
    ) -> Result<Option<OrchestrateReport>> {
        // Noop for now.
        Ok(None)
    }
}

struct Persist {
    // TODO: implement when needed.
}

impl PersistInterface for Persist {
    fn action(&self, _: Action, _: Option<SpanContext>) -> Result<()> {
        // Noop for now.
        Ok(())
    }

    fn action_history(&self, _: Vec<ActionHistory>, _: Option<SpanContext>) -> Result<()> {
        // Noop for now.
        Ok(())
    }

    fn cluster_orchestrate_report(
        &self,
        _: OrchestrateReport,
        _: Option<SpanContext>,
    ) -> Result<()> {
        // Noop for now.
        Ok(())
    }

    fn event(&self, _: Event, _: Option<SpanContext>) -> Result<()> {
        // Noop for now.
        Ok(())
    }

    fn orchestrator_action(&self, _: OrchestratorAction, _: Option<SpanContext>) -> Result<()> {
        // Noop for now.
        Ok(())
    }
}

use chrono::DateTime;
use chrono::Utc;
use opentracingrust::SpanContext;
use uuid::Uuid;

use replicante_models_core::actions::orchestrator::OrchestratorAction;

use crate::backend::OrchestratorActionsImpl;
use crate::Cursor;
use crate::Result;

/// Filters for searching actions for a cluster.
pub struct SearchFilters {
    pub action_kind: Option<String>,
    pub action_state: Option<String>,
    pub from: DateTime<Utc>,
    pub until: DateTime<Utc>,
}

/// Operate on actions.
pub struct OrchestratorActions {
    actions: OrchestratorActionsImpl,
}

impl OrchestratorActions {
    pub(crate) fn new(actions: OrchestratorActionsImpl) -> OrchestratorActions {
        OrchestratorActions { actions }
    }

    /// Fetch a specific cluster's `OrchestratorAction` record.
    pub fn orchestrator_action<S>(
        &self,
        action_id: Uuid,
        span: S,
    ) -> Result<Option<OrchestratorAction>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.actions.orchestrator_action(action_id, span.into())
    }

    /// Search a cluster for actions matching the given filters.
    pub fn search<S>(&self, filters: SearchFilters, span: S) -> Result<Cursor<OrchestratorAction>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.actions.search(filters, span.into())
    }
}

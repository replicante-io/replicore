use opentracingrust::SpanContext;
use uuid::Uuid;

use replicante_models_core::api::orchestrator_action::OrchestratorActionSummary;

use super::super::backend::OrchestratorActionsImpl;
use super::super::Cursor;
use super::super::Result;

/// Operate on all orchestrator actions in the cluster identified by cluster_id.
pub struct OrchestratorActions {
    actions: OrchestratorActionsImpl,
    attrs: OrchestratorActionsAttributes,
}

impl OrchestratorActions {
    pub(crate) fn new(
        actions: OrchestratorActionsImpl,
        attrs: OrchestratorActionsAttributes,
    ) -> OrchestratorActions {
        OrchestratorActions { actions, attrs }
    }

    /// Approve a PENDING_APPROVE orchestrator action for scheduling and execution.
    pub fn approve<S>(&self, action_id: Uuid, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.actions.approve(&self.attrs, action_id, span.into())
    }

    /// Disapprove a PENDING_APPROVE orchestrator action so it won't be scheduled.
    pub fn disapprove<S>(&self, action_id: Uuid, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.actions.disapprove(&self.attrs, action_id, span.into())
    }

    /// Iterate over orchestrator actions for a cluster.
    pub fn iter_summary<S>(&self, span: S) -> Result<Cursor<OrchestratorActionSummary>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.actions.iter_summary(&self.attrs, span.into())
    }
}

/// Attributes attached to all orchestrator actions operations.
pub struct OrchestratorActionsAttributes {
    pub cluster_id: String,
}

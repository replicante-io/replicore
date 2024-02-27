use opentracingrust::SpanContext;
use uuid::Uuid;

use replicante_models_core::actions::orchestrator::OrchestratorAction as OrchestratorActionModel;

use crate::backend::OrchestratorActionImpl;
use crate::Result;

/// Operate on the action identified by the provided cluster_id and action_id.
pub struct OrchestratorAction {
    action: OrchestratorActionImpl,
    attrs: OrchestratorActionAttributes,
}

impl OrchestratorAction {
    pub(crate) fn new(
        action: OrchestratorActionImpl,
        attrs: OrchestratorActionAttributes,
    ) -> OrchestratorAction {
        OrchestratorAction { action, attrs }
    }

    /// Return a specific `OrchestratorAction` record, if any is stored.
    pub fn get<S>(&self, span: S) -> Result<Option<OrchestratorActionModel>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.action.get(&self.attrs, span.into())
    }
}

/// Attributes attached to all `OrchestratorAction` operations.
pub struct OrchestratorActionAttributes {
    pub action_id: Uuid,
    pub cluster_id: String,
}

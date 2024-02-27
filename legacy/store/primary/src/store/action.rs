use opentracingrust::SpanContext;
use uuid::Uuid;

use replicante_models_core::actions::node::Action as ActionModel;

use crate::backend::ActionImpl;
use crate::Result;

/// Operate on the action identified by the provided cluster_id and action_id.
pub struct Action {
    action: ActionImpl,
    attrs: ActionAttributes,
}

impl Action {
    pub(crate) fn new(action: ActionImpl, attrs: ActionAttributes) -> Action {
        Action { action, attrs }
    }

    /// Return a specific `Action` record, if any is stored.
    pub fn get<S>(&self, span: S) -> Result<Option<ActionModel>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.action.get(&self.attrs, span.into())
    }
}

/// Attributes attached to all `Agent` operations.
pub struct ActionAttributes {
    pub action_id: Uuid,
    pub cluster_id: String,
}

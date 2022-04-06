use opentracingrust::SpanContext;
use uuid::Uuid;

use replicante_models_core::actions::ActionSummary;

use crate::backend::ActionsImpl;
use crate::Cursor;
use crate::Result;

/// Operate on actions for the cluster identified by cluster_id.
pub struct Actions {
    actions: ActionsImpl,
    attrs: ActionsAttributes,
}

impl Actions {
    pub(crate) fn new(actions: ActionsImpl, attrs: ActionsAttributes) -> Actions {
        Actions { actions, attrs }
    }

    /// Approve a PENDING_APPROVE action for scheduling.
    pub fn approve<S>(&self, action_id: Uuid, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.actions.approve(&self.attrs, action_id, span.into())
    }

    /// Disapprove a PENDING_APPROVE action so it won't be scheduled.
    pub fn disapprove<S>(&self, action_id: Uuid, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.actions.disapprove(&self.attrs, action_id, span.into())
    }

    /// Iterate all unfinished actions for the cluster returning only summary information.
    pub fn unfinished_summaries<S>(&self, span: S) -> Result<Cursor<ActionSummary>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.actions.unfinished_summaries(&self.attrs, span.into())
    }
}

/// Attributes attached to all `Actions` operations.
pub struct ActionsAttributes {
    pub cluster_id: String,
}

use opentracingrust::SpanContext;

use replicante_models_core::actions::node::ActionSyncSummary;

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

    /// Iterate all unfinished actions for the cluster returning only summary information.
    pub fn unfinished_summaries<S>(&self, span: S) -> Result<Cursor<ActionSyncSummary>>
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

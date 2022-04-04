use chrono::DateTime;
use chrono::Utc;
use opentracingrust::SpanContext;
use uuid::Uuid;

use replicante_models_core::actions::Action;
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

    /// Iterate over all unfinished actions on the node which were NOT updated during `refresh_id`.
    ///
    /// This method MUST return the same actions that `Actions::mark_lost` would modify.
    /// To keep callers logic simple, the `Action`s are returned as if the changes from
    /// `Actions::mark_lost` were already applied.
    pub fn iter_lost<S>(
        &self,
        node_id: String,
        refresh_id: i64,
        finished_ts: DateTime<Utc>,
        span: S,
    ) -> Result<Cursor<Action>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.actions
            .iter_lost(&self.attrs, node_id, refresh_id, finished_ts, span.into())
    }

    /// Update all unfinished actions on the node which were NOT updated during `refresh_id`.
    ///
    /// This method sets the state to `ActionState::Lost` and the finished timestamp to `Utc::now`.
    /// The method does NOT generate an action transition history record for the event.
    pub fn mark_lost<S>(
        &self,
        node_id: String,
        refresh_id: i64,
        finished_ts: DateTime<Utc>,
        span: S,
    ) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.actions
            .mark_lost(&self.attrs, node_id, refresh_id, finished_ts, span.into())
    }

    /// Iterate over all PENDING_SCHEDULE actions for the given agent/node.
    pub fn pending_schedule<S>(&self, agent_id: String, span: S) -> Result<Cursor<Action>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.actions
            .pending_schedule(&self.attrs, agent_id, span.into())
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

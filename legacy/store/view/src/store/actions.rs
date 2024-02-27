use chrono::DateTime;
use chrono::Utc;
use opentracingrust::SpanContext;
use uuid::Uuid;

use replicante_models_core::actions::node::Action;
use replicante_models_core::actions::node::ActionHistory;

use crate::backend::ActionsImpl;
use crate::Cursor;
use crate::Result;

/// Filters for searching actions for a cluster.
pub struct SearchFilters {
    pub action_kind: Option<String>,
    pub action_state: Option<String>,
    pub from: DateTime<Utc>,
    pub node_id: Option<String>,
    pub until: DateTime<Utc>,
}

/// Operate on actions.
pub struct Actions {
    actions: ActionsImpl,
}

impl Actions {
    pub(crate) fn new(actions: ActionsImpl) -> Actions {
        Actions { actions }
    }

    /// Fetch a specific cluster's `Action` record.
    pub fn action<S>(&self, action_id: Uuid, span: S) -> Result<Option<Action>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.actions.action(action_id, span.into())
    }

    /// Sets the `finished_ts` attribute on an entire action history to allow cleanup.
    pub fn finish_history<S>(
        &self,
        action_id: Uuid,
        finished_ts: DateTime<Utc>,
        span: S,
    ) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.actions
            .finish_history(action_id, finished_ts, span.into())
    }

    /// Fetch a specific cluster's `Action` history.
    ///
    /// Unlike other methods that return a `Cursor` of records, this one returns a `Vec` of them.
    /// This keeps client code simple an action histories are expected to generally be
    /// short so loading them all at once should not cause any performance penalties.
    pub fn history<S>(&self, action_id: Uuid, span: S) -> Result<Vec<ActionHistory>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.actions.history(action_id, span.into())
    }

    /// Search a cluster for actions matching the given filters.
    pub fn search<S>(&self, filters: SearchFilters, span: S) -> Result<Cursor<Action>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.actions.search(filters, span.into())
    }
}

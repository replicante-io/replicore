use std::collections::HashMap;

use opentracingrust::SpanContext;
use uuid::Uuid;

use crate::backend::ActionsImpl;
use crate::Result;

pub const MAX_ACTIONS_STATE_FOR_SYNC: usize = 20;

/// Operate on actions for the cluster identified by cluster_id.
pub struct Actions {
    actions: ActionsImpl,
    attrs: ActionsAttributes,
}

impl Actions {
    pub(crate) fn new(actions: ActionsImpl, attrs: ActionsAttributes) -> Actions {
        Actions { actions, attrs }
    }

    /// Update all unfinished actions on the node which were NOT updated during `refresh_id`.
    ///
    /// This method sets the state to `ActionState::Lost` and the finished timestamp to `Utc::now`.
    /// The method does NOT generate an action transition history record for the event.
    pub fn mark_lost<S>(&self, node_id: String, refresh_id: i64, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.actions
            .mark_lost(&self.attrs, node_id, refresh_id, span.into())
    }

    /// Return information about the given action IDs for use by the agent sync process.
    ///
    /// # Panics
    /// If the number of action IDs in `action_ids` is greater then `MAX_ACTIONS_STATE_FOR_SYNC`
    /// this method panics to ensue overly expensive `IN` queries are avoided.
    pub fn state_for_sync<S>(
        &self,
        node_id: String,
        action_ids: &[Uuid],
        span: S,
    ) -> Result<HashMap<Uuid, ActionSyncState>>
    where
        S: Into<Option<SpanContext>>,
    {
        if action_ids.len() > MAX_ACTIONS_STATE_FOR_SYNC {
            panic!("Actions::state_for_sync can check at most MAX_ACTIONS_STATE_FOR_SYNC actions at once");
        }
        self.actions
            .state_for_sync(&self.attrs, node_id, action_ids, span.into())
    }
}

/// Attributes attached to all `Actions` operations.
pub struct ActionsAttributes {
    pub cluster_id: String,
}

/// Action state in the primary store used by the agent sync process.
pub enum ActionSyncState {
    Finished,
    Found,
    NotFound,
}

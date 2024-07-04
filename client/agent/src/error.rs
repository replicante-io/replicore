//! Errors during agent interactions.
use uuid::Uuid;

/// Node action not found on node
#[derive(Debug, thiserror::Error)]
#[error("node action '{action_id}' not found on node")]
pub struct ActionNotFound {
    pub action_id: Uuid,
}

/// Cannot schedule node action because another action has the same ID
#[derive(Debug, thiserror::Error)]
#[error("cannot schedule node action '{action_id}' because another action has the same ID")]
pub struct ScheduleActionDuplicateId {
    pub action_id: Uuid,
}

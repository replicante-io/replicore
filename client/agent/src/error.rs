//! Errors during agent interactions.
use uuid::Uuid;

/// Node action not found on node
#[derive(Debug, thiserror::Error)]
#[error("node action '{action_id}' not found on node")]
pub struct ActionNotFound {
    pub action_id: Uuid,
}

use thiserror::Error;

/// Core errors during cluster orchestration operations.
///
/// These are errors in the logic or behaviour of the overall orchestration
/// operation as opposed to action/node/remote/... errors.
/// In other words these are non-recoverable errors that should end the whole operation.
#[derive(Error, Debug)]
pub enum OperationError {
    #[error("lost cluster orchestration lock for cluster {0}.{1}")]
    // (namespace_id, cluster_id)
    LockLost(String, String),
}

impl OperationError {
    /// Lock was lost since the operation was started.
    pub fn lock_lost<CID, NID>(namespace_id: NID, cluster_id: CID) -> OperationError
    where
        CID: Into<String>,
        NID: Into<String>,
    {
        OperationError::LockLost(namespace_id.into(), cluster_id.into())
    }
}

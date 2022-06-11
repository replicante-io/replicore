use thiserror::Error;

/// Error indicating an action with the given ID is already present in the registry builder.
#[derive(Error, Debug)]
#[error("orchestrator action already present in the registry: id '{id}'")]
pub struct ActionAlreadyRegistered {
    /// ID of the duplicate action being registered.
    pub id: String,
}

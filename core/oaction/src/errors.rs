//! Errors related to orchestrator action handling and definition.

/// Metadata for orchestrator action not found.
#[derive(Debug, thiserror::Error)]
#[error("metadata for orchestrator action {kind} not found")]
pub struct OActionNotFound {
    /// The orchestrator action kind being looked up.
    pub kind: String,
}

impl From<&str> for OActionNotFound {
    fn from(value: &str) -> Self {
        OActionNotFound {
            kind: value.to_string(),
        }
    }
}

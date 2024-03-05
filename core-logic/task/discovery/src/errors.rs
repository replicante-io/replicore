//! Errors encountered during platform discovery.

/// The expected platform is not in the active status.
#[derive(Debug, thiserror::Error)]
#[error("the expected platform '{ns_id}.{name}' is not in the active status")]
pub struct PlatformNotActive {
    pub ns_id: String,
    pub name: String,
}

impl PlatformNotActive {
    /// The expected platform is not in the active status.
    pub fn new<S1: Into<String>, S2: Into<String>>(ns_id: S1, name: S2) -> Self {
        Self {
            ns_id: ns_id.into(),
            name: name.into(),
        }
    }
}

/// The expected platform was not found.
#[derive(Debug, thiserror::Error)]
#[error("the expected platform '{ns_id}.{name}' was not found")]
pub struct PlatformNotFound {
    pub ns_id: String,
    pub name: String,
}

impl PlatformNotFound {
    /// The expected platform was not found.
    pub fn new<S1: Into<String>, S2: Into<String>>(ns_id: S1, name: S2) -> Self {
        Self {
            ns_id: ns_id.into(),
            name: name.into(),
        }
    }
}

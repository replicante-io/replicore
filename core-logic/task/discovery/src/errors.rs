//! Errors encountered during platform discovery.
/// The namespace is not in the active status.
#[derive(Debug, thiserror::Error)]
#[error("the namespace '{ns_id}' is not in the active status")]
pub struct NamespaceNotActive {
    pub ns_id: String,
}

impl NamespaceNotActive {
    /// The namespace is not in the active status.
    pub fn new<S: Into<String>>(ns_id: S) -> Self {
        Self {
            ns_id: ns_id.into(),
        }
    }
}

/// The expected namespace was not found.
#[derive(Debug, thiserror::Error)]
#[error("the expected namespace '{ns_id}' was not found")]
pub struct NamespaceNotFound {
    pub ns_id: String,
}

impl NamespaceNotFound {
    /// The expected namespace was not found.
    pub fn new<S: Into<String>>(ns_id: S) -> Self {
        Self {
            ns_id: ns_id.into(),
        }
    }
}

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

/// URL-based client for platform does not have a schema.
#[derive(Debug, thiserror::Error)]
#[error("URL-based client for platform '{ns_id}.{name}' does not have a schema")]
pub struct UrlClientNoSchema {
    pub ns_id: String,
    pub name: String,
}

/// Unknown schema provided for URL-based client for platform.
#[derive(Debug, thiserror::Error)]
#[error("Unknown schema provided for URL-based client for platform '{ns_id}.{name}'")]
pub struct UrlClientUnknownSchema {
    pub ns_id: String,
    pub name: String,
    pub schema: String,
}

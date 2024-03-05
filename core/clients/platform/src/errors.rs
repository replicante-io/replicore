//! Errors around Platform clients lookup or initialisation.

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

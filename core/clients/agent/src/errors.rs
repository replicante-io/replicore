//! Errors around Agent clients lookup or initialisation.

/// Client for agent does not have a schema.
#[derive(Debug, thiserror::Error)]
#[error("client for agent '{node_id}' in '{ns_id}.{cluster_id}' does not have a schema")]
pub struct ClientNoSchema {
    pub ns_id: String,
    pub cluster_id: String,
    pub node_id: String,
}

/// Unknown schema provided for agent client to node.
#[derive(Debug, thiserror::Error)]
#[error(
    "unknown schema '{schema}' provided for agent client to node '{node_id}' in '{ns_id}.{cluster_id}'"
)]
pub struct ClientUnknownSchema {
    pub ns_id: String,
    pub cluster_id: String,
    pub node_id: String,
    pub schema: String,
}

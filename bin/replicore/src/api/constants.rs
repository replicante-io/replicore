//! Constants used by API handlers.

/// Event code emitted when a ClusterSpec is deleted from the control plane.
pub const CLUSTER_SPEC_DELETED: &str = "CLUSTER_SPEC_DELETED";

/// Event code emitted when a Namespace is moved to Deleting.
pub const NAMESPACE_DELETE_REQUESTED: &str = "NAMESPACE_DELETE_REQUESTED";

/// Event code emitted when a node was updated by the API.
pub const NODE_UPDATE_API: &str = "NODE_UPDATE_API";

/// Event code emitted when a Platform is deleted from the control plane.
pub const PLATFORM_DELETED: &str = "PLATFORM_DELETED";

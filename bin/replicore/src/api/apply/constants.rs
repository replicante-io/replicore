//! Constants used by apply handlers across API versions.

/// Event code emitted when a ClusterSpec is applied (create or update).
pub const APPLY_CLUSTER_SPEC: &str = "APPLY_CLUSTER_SPEC";

/// Event code emitted when a Namespace is applied (create or update).
pub const APPLY_NAMESPACE: &str = "APPLY_NAMESPACE";

/// Event code emitted when a Platform is applied (create or update).
pub const APPLY_PLATFORM: &str = "APPLY_PLATFORM";

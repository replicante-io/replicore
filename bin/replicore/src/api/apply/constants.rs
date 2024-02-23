//! Constants used by apply handlers across API versions.

/// Event code emitted when a ClusterSpec is applied (create or update).
pub const APPLY_CLUSTER_SPEC: &str = "CLUSTER_SPEC_APPLY";

/// Event code emitted when a Namespace is applied (create or update).
pub const APPLY_NAMESPACE: &str = "NAMESPACE_APPLY";

/// Event code emitted when a Platform is applied (create or update).
pub const APPLY_PLATFORM: &str = "PLATFORM_APPLY";

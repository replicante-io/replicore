//! Constants related to cluster convergence.

/// Action Kind for cluster expansion by adding requests.
pub const ACTION_KIND_CLUSTER_ADD: &str = "agent.replicante.io/cluster.add";

/// Action Kind for cluster initialisation requests.
pub const ACTION_KIND_CLUSTER_INIT: &str = "agent.replicante.io/cluster.init";

/// Action Kind for cluster expansion by joining requests.
pub const ACTION_KIND_CLUSTER_JOIN: &str = "agent.replicante.io/cluster.join";

/// Action Kind for node provisioning requests.
pub const ACTION_KIND_PROVISION: &str = "core.replicante.io/platform.provision";

/// Step ID for the Cluster Expand convergence check.
pub const STEP_ID_CLUSTER_EXPAND: &str = "cluster-expand";

/// Step ID for the Node Scale Up convergence check.
pub const STEP_ID_CLUSTER_INIT: &str = "cluster-init";

/// Step ID for the Node Scale Up convergence check.
pub const STEP_ID_SCALE_UP: &str = "node-scale-up";

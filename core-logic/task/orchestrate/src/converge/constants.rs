//! Constants related to cluster convergence.

/// Action Kind for cluster initialisation requests.
pub const ACTION_KIND_CLUSTER_INIT: &str = "agent.replicante.io/cluster.init";

/// Action Kind for node provisioning requests.
pub const ACTION_KIND_PROVISION: &str = "core.replicante.io/platform.provision";

/// Step ID for the Node Scale Up convergence check.
pub const STEP_ID_CLUSTER_INIT: &str = "cluster-init";

/// Step ID for the Node Scale Up convergence check.
pub const STEP_ID_SCALE_UP: &str = "node-scale-up";

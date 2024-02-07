//! Implementation of cluster orchestration tasks.
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;

use replicore_tasks::conf::Queue;
use replicore_tasks::submit::TaskSubmission;

mod callback;

pub use self::callback::Callback;

/// Background task queue for cluster orchestration requests.
pub static ORCHESTRATE_QUEUE: Lazy<Queue> = Lazy::new(|| Queue {
    queue: String::from("cluster_orchestrate"),
    retry_count: 1,
    retry_timeout: std::time::Duration::from_secs(5),
});

/// Request orchestration of specified cluster.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OrchestrateCluster {
    /// ID of the namespace the cluster is defined in.
    pub ns_id: String,

    /// ID of the cluster to orchestrate.
    pub cluster_id: String,
}

impl OrchestrateCluster {
    pub fn new<S1, S2>(ns_id: S1, cluster_id: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            ns_id: ns_id.into(),
            cluster_id: cluster_id.into(),
        }
    }
}

impl TryInto<TaskSubmission> for OrchestrateCluster {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<TaskSubmission, Self::Error> {
        TaskSubmission::new(&ORCHESTRATE_QUEUE, &self)
    }
}

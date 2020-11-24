use std::str::FromStr;
use std::time::Duration;

use replicante_service_tasks::TaskQueue;

pub mod payload;

/// Enumerate all queues used in Replicante.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ReplicanteQueues {
    /// Cluster state refresh and aggregation tasks.
    ClusterRefresh,

    /// Fetch cluster `DiscoveryRecord`s from a discovery backend.
    DiscoverClusters,

    /// Orchestrate a cluster to converge to the configured desired state.
    OrchestrateCluster,
}

impl FromStr for ReplicanteQueues {
    type Err = failure::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cluster_refresh" => Ok(ReplicanteQueues::ClusterRefresh),
            "discover_clusters" => Ok(ReplicanteQueues::DiscoverClusters),
            "orchestrate_cluster" => Ok(ReplicanteQueues::OrchestrateCluster),
            s => Err(::failure::err_msg(format!("unknown queue '{}'", s))),
        }
    }
}

impl TaskQueue for ReplicanteQueues {
    fn max_retry_count(&self) -> u8 {
        match self {
            ReplicanteQueues::ClusterRefresh => 1,
            ReplicanteQueues::DiscoverClusters => 1,
            ReplicanteQueues::OrchestrateCluster => 1,
        }
    }

    fn name(&self) -> String {
        match self {
            ReplicanteQueues::ClusterRefresh => "cluster_refresh".into(),
            ReplicanteQueues::DiscoverClusters => "discover_clusters".into(),
            ReplicanteQueues::OrchestrateCluster => "orchestrate_cluster".into(),
        }
    }

    fn retry_delay(&self) -> Duration {
        match self {
            ReplicanteQueues::ClusterRefresh => Duration::from_secs(2),
            ReplicanteQueues::DiscoverClusters => Duration::from_secs(2),
            ReplicanteQueues::OrchestrateCluster => Duration::from_secs(5),
        }
    }
}

/// Type-specialised task model.
pub type Task = replicante_service_tasks::Task<ReplicanteQueues>;

/// Type-specialised task requester.
pub type Tasks = replicante_service_tasks::Tasks<ReplicanteQueues>;

/// Type-specialised task requester mock.
#[cfg(any(test, feature = "with_test_support"))]
pub type MockTasks = replicante_service_tasks::MockTasks<ReplicanteQueues>;

use std::str::FromStr;
use std::time::Duration;

use replicante_tasks::TaskQueue;

pub mod payload;

/// Enumerate all queues used in Replicante.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ReplicanteQueues {
    /// Cluster state refresh and aggregation tasks.
    ClusterRefresh,
}

impl FromStr for ReplicanteQueues {
    type Err = ::failure::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cluster_refresh" => Ok(ReplicanteQueues::ClusterRefresh),
            s => Err(::failure::err_msg(format!("unknown queue '{}'", s))),
        }
    }
}

impl TaskQueue for ReplicanteQueues {
    fn max_retry_count(&self) -> u8 {
        match self {
            ReplicanteQueues::ClusterRefresh => 3,
            //_ => 12,
        }
    }

    fn name(&self) -> String {
        match self {
            ReplicanteQueues::ClusterRefresh => "cluster_refresh".into(),
        }
    }

    fn retry_delay(&self) -> Duration {
        match self {
            ReplicanteQueues::ClusterRefresh => Duration::from_secs(10),
            //_ => Duration::from_secs(5 * 60),
        }
    }
}

/// Type-specialised task model.
pub type Task = ::replicante_tasks::Task<ReplicanteQueues>;

/// Type-specialised task requester.
pub type Tasks = ::replicante_tasks::Tasks<ReplicanteQueues>;

/// Type-specialised task requester mock.
#[cfg(test)]
pub type MockTasks = ::replicante_tasks::MockTasks<ReplicanteQueues>;

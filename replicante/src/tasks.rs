use std::str::FromStr;

use replicante_tasks::TaskQueue;


/// Enumerate all queues used in Replicante.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ReplicanteQueues {
    /// Queue used to spawn cluster discovery tasks.
    Discovery,
}

impl FromStr for ReplicanteQueues {
    type Err = ::failure::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "discovery" => Ok(ReplicanteQueues::Discovery),
            s => Err(::failure::err_msg(format!("unknown queue '{}'", s))),
        }
    }
}

impl TaskQueue for ReplicanteQueues {
    fn name(&self) -> String {
        match self {
            ReplicanteQueues::Discovery => "discovery".into(),
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

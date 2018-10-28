use replicante_tasks::TaskQueue;


/// Enumerate all queues used in Replicante.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ReplicanteQueues {
    /// Queue used to spawn cluster discovery tasks.
    Discovery,
}

impl TaskQueue for ReplicanteQueues {
    fn name(&self) -> String {
        match self {
            ReplicanteQueues::Discovery => "discovery".into(),
        }
    }
}


/// Type-specialised task requester.
pub type Tasks = ::replicante_tasks::Tasks<ReplicanteQueues>;


/// Type-specialised task requester mock.
#[cfg(debug_assertions)]
pub type MockTasks = ::replicante_tasks::MockTasks<ReplicanteQueues>;

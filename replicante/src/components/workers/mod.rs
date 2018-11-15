use slog::Logger;

use replicante_data_models::ClusterDiscovery;

use replicante_tasks::Config;
use replicante_tasks::WorkerSet;
use replicante_tasks::WorkerSetPool;

use super::super::Result;
use super::super::interfaces::Interfaces;
use super::super::tasks::ReplicanteQueues;
use super::super::tasks::Task;


/// Store the state of the WorkerSet.
///
/// Initially the pool is configured but not active.
/// When the component is run, the pool configuration is taken and a handle to
/// the workers pool is stored in its place.
enum State {
    Configured(WorkerSet<ReplicanteQueues>),
    Started(WorkerSetPool),
}

/// Wrapper object around `replicante_tasks::WorkerSet` objects.
pub struct Workers {
    state: Option<State>,
}

impl Workers {
    /// Configure the task workers to be run later.
    ///
    /// The tasks that are processed by this node are defined in the configuration file.
    pub fn new(_interfaces: &mut Interfaces, logger: Logger, config: Config) -> Result<Workers> {
        let worker_set = WorkerSet::new(logger.clone(), config)?;
        // TODO: dynamic worker configuration.
        // TODO: move tasks to external fns/modules for cleaner code.
        let worker_set = worker_set.worker(ReplicanteQueues::Discovery, move |task: Task| {
            let discovery: ClusterDiscovery = task.deserialize()?;
            debug!(logger, "TODO: implement discovery task"; "discovery" => ?discovery);
            ::std::thread::sleep(::std::time::Duration::from_secs(5));
            task.success()?;
            Ok(())
        })?;
        Ok(Workers {
            state: Some(State::Configured(worker_set)),
        })
    }

    /// Convert the WorkerSet configuration into a runnning WorkerSetPool.
    pub fn run(&mut self) -> Result<()> {
        if let Some(State::Configured(worker_set)) = self.state.take() {
            let workers = worker_set.run()?;
            self.state = Some(State::Started(workers));
            Ok(())
        } else {
            Err("workers already running".into())
        }
    }

    /// Stop worker threads and wait for them to terminate.
    pub fn wait(&mut self) -> Result<()> {
        if let Some(State::Started(pool)) = self.state.take() {
            drop(pool);
            Ok(())
        } else {
            Err("workers not running".into())
        }
    }
}

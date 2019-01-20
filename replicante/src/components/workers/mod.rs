use failure::err_msg;
use prometheus::Registry;
use slog::Logger;

use replicante_tasks::Config;
use replicante_tasks::TaskHandler;
use replicante_tasks::WorkerSet;
use replicante_tasks::WorkerSetPool;

use super::super::ErrorKind;
use super::super::Result;
use super::super::config::TaskWorkers;
use super::super::interfaces::Interfaces;
use super::super::tasks::ReplicanteQueues;
use super::super::tasks::Task;


mod cluster_refresh;


/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    self::cluster_refresh::register_metrics(logger, registry);
}


/// Store the state of the WorkerSet.
///
/// Initially the pool is configured but not active.
/// When the component is run, the pool configuration is taken and a handle to
/// the workers pool is stored in its place.
enum State {
    Configured(WorkerSet<ReplicanteQueues>),
    Started(WorkerSetPool),
}


/// Helper function to keep `Workers::new` simpler in the presence of conditional queues.
fn configure_worker<F, H>(
    workers: WorkerSet<ReplicanteQueues>, queue: ReplicanteQueues, enabled: bool, factory: F
) -> Result<WorkerSet<ReplicanteQueues>>
    where F: Fn() -> H,
          H: TaskHandler<ReplicanteQueues>,
{
    if enabled {
        let handler = factory();
        let workers = workers.worker(queue, handler)?;
        return Ok(workers);
    }
    Ok(workers)
}


/// Wrapper object around `replicante_tasks::WorkerSet` objects.
pub struct Workers {
    state: Option<State>,
}

impl Workers {
    /// Configure the task workers to be run later.
    ///
    /// The tasks that are processed by this node are defined in the configuration file.
    pub fn new(
        interfaces: &mut Interfaces, logger: Logger, config: Config, task_workers: TaskWorkers
    ) -> Result<Workers> {
        let worker_set = WorkerSet::new(logger.clone(), config)?;
        let worker_set = configure_worker(
            worker_set, ReplicanteQueues::ClusterRefresh, task_workers.cluster_refresh(),
            || self::cluster_refresh::Handler::new(interfaces, logger.clone())
        )?;
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
            Err(ErrorKind::Legacy(err_msg("workers already running")))?
        }
    }

    /// Stop worker threads and wait for them to terminate.
    pub fn wait(&mut self) -> Result<()> {
        if let Some(State::Started(pool)) = self.state.take() {
            drop(pool);
            Ok(())
        } else {
            Err(ErrorKind::Legacy(err_msg("workers not running")))?
        }
    }
}

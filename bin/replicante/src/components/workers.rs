use std::time::Duration;

use failure::ResultExt;
use prometheus::Registry;
use slog::Logger;

use replicante_service_tasks::TaskHandler;
use replicante_service_tasks::TaskQueue;
use replicante_service_tasks::WorkerSet;
use replicante_service_tasks::WorkerSetPool;
use replicante_util_upkeep::Upkeep;

use replicore_models_tasks::ReplicanteQueues;

use super::Component;
use crate::config::Config;
use crate::interfaces::Interfaces;
use crate::metrics::WORKERS_ENABLED;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    replicore_task_discovery::register_metrics(logger, registry);
    replicore_task_orchestrator::register_metrics(logger, registry);
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
    workers: WorkerSet<ReplicanteQueues>,
    queue: ReplicanteQueues,
    enabled: bool,
    factory: F,
) -> Result<WorkerSet<ReplicanteQueues>>
where
    F: Fn() -> H,
    H: TaskHandler<ReplicanteQueues>,
{
    let name = queue.name();
    if enabled {
        WORKERS_ENABLED.with_label_values(&[&name]).set(1.0);
        let handler = factory();
        let workers = match workers.worker(queue, handler) {
            Ok(workers) => workers,
            Err(error) => {
                return Err(error)
                    .with_context(|_| ErrorKind::TaskWorkerRegistration(name))
                    .map_err(Error::from)
            }
        };
        return Ok(workers);
    }
    WORKERS_ENABLED.with_label_values(&[&name]).set(0.0);
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
    pub fn new(interfaces: &mut Interfaces, logger: Logger, config: Config) -> Result<Workers> {
        let worker_set = WorkerSet::new(
            logger.clone(),
            config.tasks.clone(),
            interfaces.healthchecks.register(),
        )
        .with_context(|_| ErrorKind::ClientInit("tasks workers"))?;
        let worker_set = configure_worker(
            worker_set,
            ReplicanteQueues::DiscoverClusters,
            config.task_workers.discover_clusters(),
            || {
                replicore_task_discovery::DiscoverClusters::new(
                    interfaces.streams.events.clone(),
                    logger.clone(),
                    interfaces.stores.primary.clone(),
                    interfaces.tracing.tracer(),
                )
            },
        )?;
        let worker_set = configure_worker(
            worker_set,
            ReplicanteQueues::OrchestrateCluster,
            config.task_workers.orchestrate_cluster(),
            || {
                replicore_task_orchestrator::OrchestrateCluster::new(
                    Duration::from_secs(config.timeouts.agents_api),
                    interfaces.coordinator.clone(),
                    interfaces.streams.events.clone(),
                    logger.clone(),
                    interfaces.stores.primary.clone(),
                    interfaces.tracing.tracer(),
                )
            },
        )?;
        Ok(Workers {
            state: Some(State::Configured(worker_set)),
        })
    }
}

impl Component for Workers {
    /// Convert the WorkerSet configuration into a runnning WorkerSetPool.
    fn run(&mut self, upkeep: &mut Upkeep) -> Result<()> {
        if let Some(State::Configured(worker_set)) = self.state.take() {
            let workers = worker_set
                .run(upkeep)
                .with_context(|_| ErrorKind::ThreadSpawn("tasks workers"))?;
            self.state = Some(State::Started(workers));
            Ok(())
        } else {
            Err(ErrorKind::ComponentAlreadyRunning("workers").into())
        }
    }
}

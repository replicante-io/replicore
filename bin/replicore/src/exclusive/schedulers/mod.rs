//! Exclusive Control Plane tasks for scheduling of other tasks.
use replisdk::runtime::shutdown::ShutdownManagerBuilder;

use replicore_conf::ExclusivesConf;
use replicore_coordinator::CoordinatorBuilder;
use replicore_injector::Injector;

mod discovery;
mod orchestrator;

use self::discovery::Discovery;
use self::orchestrator::Orchestrate;

/// Register periodic scheduling tasks with the distributed Coordinator instance.
pub fn register_tasks(
    coordinator: CoordinatorBuilder,
    conf: &ExclusivesConf,
    injector: &Injector,
    shutdown: &ShutdownManagerBuilder<()>,
) -> CoordinatorBuilder {
    coordinator
        .task(Discovery::task(
            conf.schedulers.platform_discovery,
            injector.store.clone(),
            injector.tasks.clone(),
            shutdown.shutdown_handle(),
        ))
        .task(Orchestrate::task(
            conf.schedulers.cluster_orchestrator,
            injector.store.clone(),
            injector.tasks.clone(),
            shutdown.shutdown_handle(),
        ))
}

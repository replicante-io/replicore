//! Exclusive Control Plane tasks to handle backends maintenance.
use replisdk::runtime::shutdown::ShutdownManagerBuilder;

use replicore_conf::ExclusivesConf;
use replicore_coordinator::CoordinatorBuilder;
use replicore_injector::Injector;

mod coordinator;
mod events;

use self::coordinator::Coordinator;
use self::events::Events;

/// Register maintenance tasks with the distributed Coordinator instance.
pub fn register_tasks(
    coordinator: CoordinatorBuilder,
    conf: &ExclusivesConf,
    injector: &Injector,
    shutdown: &ShutdownManagerBuilder<()>,
) -> CoordinatorBuilder {
    coordinator
        .task(Coordinator::task(
            conf.maintenance.coordinator,
            injector.leases.clone(),
            shutdown.shutdown_handle(),
        ))
        .task(Events::task(
            conf.maintenance.events,
            injector.events.clone(),
            shutdown.shutdown_handle(),
        ))
}

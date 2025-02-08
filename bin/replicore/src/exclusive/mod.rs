//! Module for exclusive Control Plane logic.
use anyhow::Result;

use replisdk::runtime::shutdown::ShutdownManagerBuilder;

use replicore_context::ContextBuilder;
use replicore_coordinator::Coordinator;
use replicore_coordinator::LeaseHandle;
use replicore_injector::Injector;

mod maintenance;
mod schedulers;

/// Start a tokio task executing a [`Coordinator`] for the process.
///
/// If this process is excluded from exclusive tasks execution, no coordinator task is created.
///
/// While this requires a single node to take either all tasks or none, it keeps the system
/// complexity much lower than each exclusive task Coordinated individually.
pub async fn component(
    context: ContextBuilder,
    injector: &Injector,
    shutdown: &mut ShutdownManagerBuilder<()>,
) -> Result<Option<LeaseHandle>> {
    // Customise the root context for the tasks executor.
    let context = context.log_values(slog::o!("component" => "tasks")).build();

    // Skip the exclusive tasks component if configured to do so.
    let conf = &injector.conf.exclusives;
    if !conf.candidate {
        slog::info!(
            context.logger,
            "Process is not a candidate for exclusive tasks, skipping coordinator"
        );
        return Ok(None);
    }

    // Build the coordinator set.
    let coordinator = Coordinator::builder(&context, &injector.leases)
        .await?
        .task(self::maintenance::Coordinator::task(
            conf.maintenance.coordinator,
            injector.leases.clone(),
            shutdown.shutdown_handle(),
        ))
        .task(self::schedulers::Discovery::task(
            conf.schedulers.platform_discovery,
            injector.store.clone(),
            injector.tasks.clone(),
            shutdown.shutdown_handle(),
        ));

    // Spawn the task if at least one task was registered.
    if coordinator.is_empty() {
        slog::warn!(
            context.logger,
            "Exclusive tasks coordinator has no registered work, skipping"
        );
        return Ok(None);
    }

    // Execute the coordinator in the background until shutdown.
    let handle = coordinator.handle();
    let coordinator = coordinator.build();
    let exit = shutdown.shutdown_handle();
    shutdown.watch_tokio(tokio::spawn(async move {
        slog::info!(context.logger, "Started exclusive tasks coordinator");
        coordinator.run(&context, exit).await
    }));
    Ok(Some(handle))
}

//! Distributed coordinator exclusive maintenance task.
use std::time::Duration;

use replisdk::runtime::shutdown::ShutdownHandle;

use replicore_context::Context;
use replicore_coordinator::ICoordinatedTimer;
use replicore_coordinator::LeaseRegistry;
use replicore_coordinator::Timer;

/// Periodically perform distributed coordinator maintenance.
pub struct Coordinator {
    registry: LeaseRegistry,
}

impl Coordinator {
    /// Initialise a timer to periodically perform maintenance.
    pub fn task(interval: u64, registry: LeaseRegistry, exit: ShutdownHandle) -> Timer<Self> {
        let coordinator = Coordinator { registry };
        let interval = Duration::from_secs(interval);
        Timer::new(coordinator, interval, exit)
    }
}

#[async_trait::async_trait]
impl ICoordinatedTimer for Coordinator {
    async fn tick(&self, context: &Context) {
        slog::debug!(
            context.logger,
            "Starting Distributed Coordinator maintenance"
        );
        if let Err(error) = self.registry.maintenance(context).await {
            slog::error!(
                context.logger, "Distributed Coordinator maintenance failed";
                replisdk::utils::error::slog::ErrorAttributes::from(&error),
            );
        }
    }
}

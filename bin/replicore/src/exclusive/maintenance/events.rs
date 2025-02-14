//! Events backend exclusive maintenance task.
use std::time::Duration;

use replisdk::runtime::shutdown::ShutdownHandle;

use replicore_context::Context;
use replicore_coordinator::ICoordinatedTimer;
use replicore_coordinator::Timer;
use replicore_events::emit::Events as EventsBackend;

/// Periodically perform distributed coordinator maintenance.
pub struct Events {
    events: EventsBackend,
}

impl Events {
    /// Initialise a timer to periodically perform maintenance.
    pub fn task(interval: u64, events: EventsBackend, exit: ShutdownHandle) -> Timer<Self> {
        let events = Events { events };
        let interval = Duration::from_secs(interval);
        Timer::new(events, interval, exit)
    }
}

#[async_trait::async_trait]
impl ICoordinatedTimer for Events {
    async fn tick(&self, context: &Context) {
        slog::debug!(context.logger, "Starting Events Backend maintenance");
        if let Err(error) = self.events.maintenance(context).await {
            slog::error!(
                context.logger, "Events Backend maintenance failed";
                replisdk::utils::error::slog::ErrorAttributes::from(&error),
            );
        }
    }
}

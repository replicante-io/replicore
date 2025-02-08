//! Schedule platform discovery tasks as needed.
use std::time::Duration;

use anyhow::Result;
use futures_util::TryStreamExt;

use replisdk::runtime::shutdown::ShutdownHandle;

use replicore_context::Context;
use replicore_coordinator::ICoordinatedTimer;
use replicore_coordinator::Timer;
use replicore_store::Store;
use replicore_tasks::submit::Tasks;

/// Periodically schedule Platform Discovery Operations.
pub struct Discovery {
    store: Store,
    tasks: Tasks,
}

impl Discovery {
    /// Initialise a timer to periodically schedule platform discoveries.
    pub fn task(interval: u64, store: Store, tasks: Tasks, exit: ShutdownHandle) -> Timer<Self> {
        let discovery = Discovery { store, tasks };
        let interval = Duration::from_secs(interval);
        Timer::new(discovery, interval, exit)
    }

    /// Timer logic with error returning support.
    async fn tick_checked(&self, context: &Context) -> Result<()> {
        // List platforms to run discovery for.
        let op = replicore_store::query::PlatformsPendingDiscovery;
        let mut platforms = self.store.query(context, op).await?;

        // Submit a task for each platform to discover.
        while let Some(platform) = platforms.try_next().await? {
            let task = replicore_task_discovery::DiscoverPlatform {
                ns_id: platform.ns_id.clone(),
                name: platform.name.clone(),
            };
            self.tasks.submit(context, task).await?;

            let op = replicore_store::persist::PlatformDiscovered::from(platform);
            self.store.persist(context, op).await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl ICoordinatedTimer for Discovery {
    async fn tick(&self, context: &Context) {
        slog::debug!(context.logger, "Starting Platform Discovery scheduler");
        if let Err(error) = self.tick_checked(context).await {
            slog::error!(
                context.logger, "Platform Discovery scheduling failed";
                replisdk::utils::error::slog::ErrorAttributes::from(&error),
            );
        }
    }
}

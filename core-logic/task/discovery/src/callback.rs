//! Callback invoked when platform discovery task need to be executed.
use anyhow::Result;

use replicore_context::Context;
use replicore_injector::Injector;
use replicore_tasks::execute::ReceivedTask;
use replicore_tasks::execute::TaskCallback;

use crate::DiscoverPlatform;

/// Callback to execute platform discovery tasks.
pub struct Callback {
    pub(crate) injector: Injector,
}

impl Default for Callback {
    fn default() -> Self {
        let injector = Injector::global();
        Self { injector }
    }
}

#[async_trait::async_trait]
impl TaskCallback for Callback {
    async fn execute(&self, context: &Context, task: &ReceivedTask) -> Result<()> {
        let request: DiscoverPlatform = task.decode()?;
        let ns_id = request.ns_id.clone();
        let name = request.name.clone();
        slog::debug!(
            context.logger, "Reached platform discovery task callback";
            "ns_id" => &ns_id,
            "name" => &name,
            "task_id" => &task.id,
        );

        let lock_id = format!("lock.platform.discovery.{}.{}", ns_id, name);
        let lock = self.injector.leases.lease(context, lock_id).await?;
        let work = crate::discover::discover(context, self, request);
        let result = replicore_coordinator::locked(lock, work).await;

        match result {
            Ok(result) => result,
            Err(error) if error.is::<replicore_coordinator::LockAbandoned>() => {
                slog::info!(
                    context.logger, "Platform discovery lock lost or not available";
                    "ns_id" => ns_id,
                    "name" => name,
                );
                Ok(())
            }
            Err(error) => Err(error),
        }
    }
}

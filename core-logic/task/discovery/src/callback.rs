//! Callback invoked when platform discovery task need to be executed.
use anyhow::Result;

use replicore_context::Context;
use replicore_tasks::execute::TaskCallback;
use replicore_tasks::execute::ReceivedTask;

use super::DiscoverPlatform;

/// Callback to execute platform discovery tasks.
pub struct Callback;

#[async_trait::async_trait]
impl TaskCallback for Callback {
    async fn execute(&self, context: &Context, task: &ReceivedTask) -> Result<()> {
        let request: DiscoverPlatform = task.decode()?;
        slog::debug!(
            context.logger, "Reached platform discovery task callback";
            "request" => ?request,
        );
        // TODO: really implement task execution
        Ok(())
    }
}

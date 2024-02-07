//! Callback invoked when cluster orchestration task need to be executed.
use anyhow::Result;

use replicore_context::Context;
use replicore_tasks::execute::ReceivedTask;
use replicore_tasks::execute::TaskCallback;

use super::OrchestrateCluster;

/// Callback to execute cluster orchestration tasks.
pub struct Callback;

#[async_trait::async_trait]
impl TaskCallback for Callback {
    async fn execute(&self, context: &Context, task: &ReceivedTask) -> Result<()> {
        let request: OrchestrateCluster = task.decode()?;
        slog::debug!(
            context.logger, "Reached cluster orchestration task callback";
            "request" => ?request,
        );
        // TODO: really implement task execution
        Ok(())
    }
}

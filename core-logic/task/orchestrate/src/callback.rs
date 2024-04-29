//! Callback invoked when cluster orchestration task need to be executed.
use anyhow::Result;

use replicore_context::Context;
use replicore_injector::Injector;
use replicore_tasks::execute::ReceivedTask;
use replicore_tasks::execute::TaskCallback;

use super::OrchestrateCluster;

/// Callback to execute cluster orchestration tasks.
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
        let request: OrchestrateCluster = task.decode()?;
        slog::debug!(
            context.logger, "Reached cluster orchestration task callback";
            "request" => ?request,
        );
        // TODO: lock check to avoid concurrent execution.

        // Initialise orchestration task.
        let data = crate::init::InitData::load(context, self.injector.clone(), request).await?;
        let mut cluster_new = data.cluster_current.new_build()?;

        // TODO: Sync cluster nodes and build current cluster view.

        // Process orchestrator actions.
        crate::oaction::progress(context, &data, &mut cluster_new).await?;
        // TODO: schedule new

        // Process convergence steps.
        let data = crate::converge::ConvergeData::convert(context, data).await?;
        crate::converge::run(context, &data).await?;

        // TODO: Emit the report event.
        // TODO: Persist report to store.
        Ok(())
    }
}

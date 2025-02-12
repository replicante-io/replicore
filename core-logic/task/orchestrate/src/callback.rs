//! Callback invoked when cluster orchestration task need to be executed.
use anyhow::Result;

use replicore_context::Context;
use replicore_events::Event;
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
        let ns_id = request.ns_id.clone();
        let cluster_id = request.cluster_id.clone();
        slog::debug!(
            context.logger, "Reached cluster orchestration task callback";
            "ns_id" => &ns_id,
            "cluster_id" => &cluster_id,
            "task_id" => &task.id,
        );

        let lock_id = format!("lock.cluster.orchestrate.{}.{}", ns_id, cluster_id);
        let lock = self.injector.leases.lease(context, lock_id).await?;
        let work = orchestrate(context, self, request);
        let result = replicore_coordinator::locked(lock, work).await;

        match result {
            Ok(result) => result,
            Err(error) if error.is::<replicore_coordinator::LockAbandoned>() => {
                slog::info!(
                    context.logger, "Cluster orchestration lock lost or not available";
                    "ns_id" => &ns_id,
                    "cluster_id" => &cluster_id,
                );
                Ok(())
            }
            Err(error) => Err(error),
        }
    }
}

/// Process cluster orchestrate requests.
async fn orchestrate(
    context: &Context,
    callback: &Callback,
    request: OrchestrateCluster,
) -> Result<()> {
    // Initialise orchestration task.
    let data = crate::init::InitData::load(context, callback.injector.clone(), request).await?;
    let data = crate::sync::SyncData::convert(data)?;

    // Sync cluster nodes and build current cluster view.
    crate::sync::nodes(context, &data).await?;
    crate::naction::schedule(context, &data).await?;

    // Process orchestrator actions.
    let oactions = data
        .cluster_current
        .oactions_unfinished
        .iter()
        .map(|action| (**action).clone())
        .collect::<Vec<_>>();
    let oactions = crate::oaction::progress(context, &data, oactions).await?;
    let oactions = crate::oaction::schedule(context, &data, oactions).await?;
    for action in oactions {
        data.cluster_new_mut().oaction(action)?;
    }

    // Process convergence steps unless we are observing only.
    let data = crate::converge::ConvergeData::convert(context, data).await?;
    if !matches!(
        data.mode,
        replicore_cluster_models::OrchestrateMode::Observe
    ) {
        crate::converge::run(context, &data).await?;
    }

    // Emit the report as an event and save it to store.
    let report = data
        .report
        .into_inner()
        .expect("orchestrate task report lock poisoned");
    let event = Event::new_with_payload(crate::constants::ORCHESTRATE_REPORT, &report)?;
    data.injector.events.change(context, event).await?;
    data.injector.store.persist(context, report).await?;

    Ok(())
}

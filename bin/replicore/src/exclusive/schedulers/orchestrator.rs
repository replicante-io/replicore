//! Schedule cluster orchestration tasks as needed.
use std::time::Duration;

use anyhow::Result;
use futures_util::TryStreamExt;

use replisdk::runtime::shutdown::ShutdownHandle;

use replicore_context::Context;
use replicore_coordinator::ICoordinatedTimer;
use replicore_coordinator::Timer;
use replicore_store::Store;
use replicore_tasks::submit::Tasks;

/// Periodically schedule Cluster Orchestration operations.
pub struct Orchestrate {
    store: Store,
    tasks: Tasks,
}

impl Orchestrate {
    /// Initialise a timer to periodically schedule platform discoveries.
    pub fn task(interval: u64, store: Store, tasks: Tasks, exit: ShutdownHandle) -> Timer<Self> {
        let orchestrate = Orchestrate { store, tasks };
        let interval = Duration::from_secs(interval);
        Timer::new(orchestrate, interval, exit)
    }

    /// Timer logic with error returning support.
    async fn tick_checked(&self, context: &Context) -> Result<()> {
        // List clusters to run orchestration for.
        let op = replicore_store::query::ClustersPendingOrchestration;
        let mut clusters = self.store.query(context, op).await?;

        // Submit a task for each cluster to orchestrate.
        while let Some(cluster) = clusters.try_next().await? {
            let task = replicore_task_orchestrate::OrchestrateCluster {
                ns_id: cluster.ns_id.clone(),
                cluster_id: cluster.cluster_id.clone(),
            };
            self.tasks.submit(context, task).await?;
            let op = replicore_store::persist::ClusterOrchestrated::from(cluster);
            self.store.persist(context, op).await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl ICoordinatedTimer for Orchestrate {
    async fn tick(&self, context: &Context) {
        slog::debug!(context.logger, "Starting Cluster Orchestration scheduler");
        if let Err(error) = self.tick_checked(context).await {
            slog::error!(
                context.logger, "Cluster Orchestration scheduling failed";
                replisdk::utils::error::slog::ErrorAttributes::from(&error),
            );
        }
    }
}

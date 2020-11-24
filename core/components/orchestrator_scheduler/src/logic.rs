use std::sync::Arc;

use failure::ResultExt;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use slog::debug;
use slog::Logger;

use replicante_models_core::cluster::ClusterSettings;
use replicante_service_tasks::TaskRequest;
use replicante_store_primary::store::Store;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_tracing::fail_span;

use replicore_models_tasks::payload::OrchestrateClusterPayload;
use replicore_models_tasks::ReplicanteQueues;
use replicore_models_tasks::Tasks;

use crate::metrics::SCHEDULE_COUNT;
use crate::ErrorKind;
use crate::Result;

/// Handle fetching and scheduling cluster discovery tasks.
pub struct Logic {
    logger: Logger,
    store: Store,
    tasks: Tasks,
    tracer: Arc<Tracer>,
}

impl Logic {
    pub fn new(logger: Logger, store: Store, tasks: Tasks, tracer: Arc<Tracer>) -> Logic {
        Logic {
            logger,
            store,
            tasks,
            tracer,
        }
    }

    /// Search for pending cluster orchestration tasks and schedule them.
    ///
    /// Update the next_orchestrate attribute when the orchestration is scheduled.
    /// This prevents scheduling the same orchestration repetitively in many situations:
    ///  * Slow or busy workers may fail to keep up (adding more work won't help).
    ///  * Incorrect configuration (short discovery loop intervals).
    ///  * One of many many possible bugs ...
    pub fn run(&self) -> Result<()> {
        let mut span = self
            .tracer
            .span("component.orchestrate_clusters")
            .auto_finish();
        let span_context = span.context().clone();
        let clusters = self
            .store
            .global_search()
            .clusters_to_orchestrate(span_context.clone())
            .context(ErrorKind::ClustersSearch)
            .map_err(|error| fail_span(error, &mut *span))?;

        for cluster in clusters {
            self.schedule_orchestrate(cluster, span_context.clone())
                .map_err(|error| fail_span(error, &mut *span))?;
            SCHEDULE_COUNT.inc();
        }
        Ok(())
    }

    /// Process an individual ClusterSettings record and schedule a discovery task for it.
    fn schedule_orchestrate(
        &self,
        cluster: replicante_store_primary::Result<ClusterSettings>,
        span_context: SpanContext,
    ) -> Result<()> {
        let cluster = cluster.context(ErrorKind::ClustersPartialSearch)?;
        debug!(
            self.logger,
            "Scheduling pending cluster orchestration";
            "namespace" => &cluster.namespace,
            "name" => &cluster.cluster_id,
        );

        // Enqueue cluster orchestration task.
        let namespace = cluster.namespace.clone();
        let cluster_id = cluster.cluster_id.clone();
        let payload = OrchestrateClusterPayload::new(&namespace, &cluster_id);
        let mut task = TaskRequest::new(ReplicanteQueues::OrchestrateCluster);
        if let Err(error) = task.trace(&span_context, &self.tracer) {
            let error = failure::SyncFailure::new(error);
            capture_fail!(
                &error,
                self.logger,
                "Unable to inject trace context in task request";
                "namespace" => &namespace,
                "cluster_id" => &cluster_id,
                failure_info(&error),
            );
        }
        if let Err(error) = self.tasks.request(task, payload) {
            capture_fail!(
                &error,
                self.logger,
                "Failed to request cluster orchestration";
                "namespace" => &namespace,
                "cluster_id" => &cluster_id,
                failure_info(&error),
            );
        };

        // Update next_orchestrate attribute so we don't spam ourselves with tasks.
        self.store
            .persist()
            .next_cluster_orchestrate(cluster, span_context)
            .with_context(|_| ErrorKind::persist_next_orchestrate(namespace, cluster_id))?;
        Ok(())
    }
}

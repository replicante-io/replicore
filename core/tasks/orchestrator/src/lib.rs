//! Implmentation of the cluster orchestration process.
use std::sync::Arc;
use std::time::Duration;

use failure::Fail;
use failure::ResultExt;
use opentracingrust::Span;
use opentracingrust::Tracer;
use slog::debug;
use slog::info;
use slog::Logger;

use replicante_service_coordinator::Coordinator;
use replicante_service_coordinator::ErrorKind as CoordinatorErrorKind;
use replicante_service_tasks::TaskHandler;
use replicante_store_primary::store::Store;
use replicante_stream_events::Stream;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_tracing::fail_span;

use replicore_models_tasks::payload::OrchestrateClusterPayload;
use replicore_models_tasks::ReplicanteQueues;
use replicore_models_tasks::Task;

mod error;
mod logic;
mod metrics;

pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::metrics::register_metrics;

use self::logic::Logic;
use self::metrics::ORCHESTRATE_DURATION;
use self::metrics::ORCHESTRATE_LOCKED;

/// Task handler for `ReplicanteQueues::OrchestrateCluster` tasks.
pub struct OrchestrateCluster {
    coordinator: Coordinator,
    logic: Logic,
    logger: Logger,
    tracer: Arc<Tracer>,
}

impl OrchestrateCluster {
    pub fn new(
        agents_timeout: Duration,
        coordinator: Coordinator,
        events: Stream,
        logger: Logger,
        store: Store,
        tracer: Arc<Tracer>,
    ) -> OrchestrateCluster {
        let logic = Logic::new(
            agents_timeout,
            events,
            logger.clone(),
            store,
            Arc::clone(&tracer),
        );
        OrchestrateCluster {
            coordinator,
            logic,
            logger,
            tracer,
        }
    }

    fn handle_task(&self, task: &Task, span: &mut Span) -> Result<()> {
        let payload: OrchestrateClusterPayload =
            task.deserialize().context(ErrorKind::DeserializePayload)?;
        let namespace = payload.namespace;
        let cluster_id = payload.cluster_id;
        span.tag("cluster.namespace", namespace.clone());
        span.tag("cluster.id", cluster_id.clone());

        // Ensure only one orchestration task is running for the same cluster.
        let span_context = span.context().clone();
        let mut lock = self
            .coordinator
            .non_blocking_lock(format!("orchestrate_cluster/{}.{}", namespace, cluster_id));
        match lock.acquire(span_context.clone()) {
            Ok(()) => (),
            Err(error) => {
                if let CoordinatorErrorKind::LockHeld(_, owner) = error.kind() {
                    ORCHESTRATE_LOCKED.inc();
                    info!(
                        self.logger,
                        "Skipped cluster orchestration because another task is in progress";
                        "namespace" => &namespace,
                        "cluster_id" => &cluster_id,
                        "owner" => %owner,
                    );
                    span.tag("coordinator.lock.busy", true);
                    return Ok(());
                }
                let error =
                    error.context(ErrorKind::concurrent_orchestrate(&namespace, &cluster_id));
                return Err(error.into());
            }
        };

        // Proceed to orchestrate the cluster while we hold the lock.
        debug!(
            self.logger,
            "Orchestrating cluster";
            "namespace" => &namespace,
            "cluster_id" => &cluster_id,
        );
        let timer = ORCHESTRATE_DURATION.start_timer();
        self.logic
            .orchestrate(&namespace, &cluster_id, &lock, span)?;
        timer.observe_duration();

        // Done.
        lock.release(span_context)
            .context(ErrorKind::release_lock(&namespace, &cluster_id))?;
        debug!(
            self.logger,
            "Cluster orchestration complete";
            "namespace" => namespace,
            "cluster_id" => cluster_id,
        );
        Ok(())
    }
}

impl TaskHandler<ReplicanteQueues> for OrchestrateCluster {
    fn handle(&self, task: Task) {
        let mut span = self.tracer.span("task.orchestrate_cluster").auto_finish();
        // If the task is carring a tracing context set it as the parent span.
        match task.trace(&self.tracer) {
            Ok(Some(parent)) => span.follows(parent),
            Ok(None) => (),
            Err(error) => {
                let error = failure::SyncFailure::new(error);
                capture_fail!(
                    &error,
                    self.logger,
                    "Unable to extract trace context from task";
                    failure_info(&error),
                );
            }
        };
        let result = self
            .handle_task(&task, &mut span)
            .map_err(|error| fail_span(error, &mut *span));
        match result {
            Ok(()) => {
                if let Err(error) = task.success() {
                    capture_fail!(
                        &error,
                        self.logger,
                        "Error while acking successfully processed task";
                        failure_info(&error),
                    );
                }
            }
            Err(error) => {
                capture_fail!(
                    &error,
                    self.logger,
                    "Failed to handle cluster orchestration task";
                    failure_info(&error),
                );
                if let Err(error) = task.fail() {
                    capture_fail!(
                        &error,
                        self.logger,
                        "Error while acking failed task";
                        failure_info(&error),
                    );
                }
            }
        }
    }
}

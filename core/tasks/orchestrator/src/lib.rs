//! Implementation of the cluster orchestration process.
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use failure::Fail;
use failure::ResultExt;
use opentracingrust::Span;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use slog::debug;
use slog::info;
use slog::Logger;

use replicante_models_core::cluster::OrchestrateReport;
use replicante_models_core::cluster::OrchestrateReportBuilder;
use replicante_models_core::events::Event;
use replicante_service_coordinator::Coordinator;
use replicante_service_coordinator::ErrorKind as CoordinatorErrorKind;
use replicante_service_tasks::TaskHandler;
use replicante_store_primary::store::Store;
use replicante_stream_events::EmitMessage;
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
    events: Stream,
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
            events.clone(),
            logger.clone(),
            store,
            Arc::clone(&tracer),
        );
        OrchestrateCluster {
            coordinator,
            events,
            logic,
            logger,
            tracer,
        }
    }

    /// Handle emitting an `OrchestrateReport` event to aid operational support.
    fn emit_report(
        &self,
        report: anyhow::Result<OrchestrateReport>,
        span: &SpanContext,
    ) -> anyhow::Result<()> {
        let report = report?;
        let event = Event::builder().cluster().orchestrate_report(report);
        let stream_key = event.stream_key();
        let event = EmitMessage::with(stream_key, event)
            .map_err(|error| anyhow::Error::new(error.compat()))
            .context("unable to serialise event payload")?
            .trace(span.clone());
        self.events
            .emit(event)
            .map_err(|error| anyhow::Error::new(error.compat()))
            .context("unable to emit orchestrate report")?;
        Ok(())
    }

    fn handle_task(
        &self,
        task: &Task,
        report: &mut OrchestrateReportBuilder,
        span: &mut Span,
    ) -> Result<()> {
        let payload: OrchestrateClusterPayload =
            task.deserialize().context(ErrorKind::DeserializePayload)?;
        let namespace = payload.namespace;
        let cluster_id = payload.cluster_id;
        span.tag("cluster.namespace", namespace.clone());
        span.tag("cluster.id", cluster_id.clone());
        report.for_cluster(&namespace, &cluster_id);

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
        report.start_now();
        let timer = ORCHESTRATE_DURATION.start_timer();
        self.logic
            .orchestrate(&namespace, &cluster_id, report, &lock, span)?;
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
        // If the task is carrying a tracing context set it as the parent span.
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

        // Start an orchestrate report for debugging this task.
        let mut report = OrchestrateReportBuilder::new();

        // Handle the task.
        let result = self
            .handle_task(&task, &mut report, &mut span)
            .map_err(|error| fail_span(error, &mut *span));

        // Finish the report and emit it as an event.
        report.outcome(crate::error::orchestrate_outcome(&result));
        let report = report.build();

        // Ack or nack the task execution.
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

        // Emit orchestrate reports.
        // Errors building or emitting orchestrate reports are logged but we don't
        // want to impact system functionality on debugging data.
        if let Err(error) = self.emit_report(report, span.context()) {
            debug!(
                self.logger,
                "Unable to emit orchestrate report";
                "cause" => #?error,
            );
        }
    }
}

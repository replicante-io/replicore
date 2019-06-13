use std::time::Duration;

use failure::Fail;
use failure::ResultExt;
use opentracingrust::Log;
use opentracingrust::Span;
use slog::debug;
use slog::info;
use slog::Logger;

use replicante_cluster_aggregator::Aggregator;
use replicante_cluster_fetcher::Fetcher;
use replicante_cluster_fetcher::Snapshotter;
use replicante_data_store::store::Store;
use replicante_models_core::ClusterDiscovery;
use replicante_models_core::Event;
use replicante_service_coordinator::Coordinator;
use replicante_service_coordinator::ErrorKind as CoordinatorErrorKind;
use replicante_service_tasks::TaskHandler;
use replicante_streams_events::EventsStream;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_tracing::fail_span;

use super::super::interfaces::tracing::Tracing;
use super::super::ErrorKind;
use super::super::Interfaces;
use super::super::Result;
use super::payload::ClusterRefreshPayload;
use super::ReplicanteQueues;
use super::Task;

mod metrics;

pub use self::metrics::register_metrics;
use self::metrics::REFRESH_DURATION;
use self::metrics::REFRESH_LOCKED;

/// Task handler for `ReplicanteQueues::Discovery` tasks.
pub struct Handler {
    aggregator: Aggregator,
    coordinator: Coordinator,
    events: EventsStream,
    fetcher: Fetcher,
    logger: Logger,
    store: Store,
    tracing: Tracing,
}

impl Handler {
    pub fn new(interfaces: &Interfaces, logger: Logger, agents_timeout: Duration) -> Handler {
        let aggregator = Aggregator::new(logger.clone(), interfaces.store.clone());
        let coordinator = interfaces.coordinator.clone();
        let events = interfaces.streams.events.clone();
        let fetcher = Fetcher::new(
            logger.clone(),
            interfaces.streams.events.clone(),
            interfaces.store.clone(),
            agents_timeout,
            interfaces.tracing.tracer(),
        );
        let store = interfaces.store.clone();
        let tracing = interfaces.tracing.clone();
        Handler {
            aggregator,
            coordinator,
            events,
            fetcher,
            logger,
            store,
            tracing,
        }
    }

    fn do_handle(&self, task: &Task, span: &mut Span) -> Result<()> {
        let payload: ClusterRefreshPayload = task
            .deserialize()
            .with_context(|_| ErrorKind::Deserialize("task payload", "ClusterRefreshPayload"))?;
        let discovery = payload.cluster;
        let snapshot = payload.snapshot;
        span.tag("cluster.id", discovery.cluster_id.clone());
        span.tag("emit.snapshot", snapshot);

        // Ensure only one refresh at the same time.
        let mut lock = self
            .coordinator
            .non_blocking_lock(format!("cluster_refresh/{}", discovery.cluster_id));
        match lock.acquire(span.context().clone()) {
            Ok(()) => (),
            Err(error) => {
                if let CoordinatorErrorKind::LockHeld(_, owner) = error.kind() {
                    REFRESH_LOCKED.inc();
                    info!(
                        self.logger,
                        "Skipped cluster refresh because another task is in progress";
                        "cluster_id" => discovery.cluster_id,
                        "owner" => %owner
                    );
                    span.tag("coordinator.lock.busy", true);
                    return Ok(());
                }
                return Err(error.context(ErrorKind::Coordination).into());
            }
        };

        // Refresh cluster state.
        let cluster_id = discovery.cluster_id.clone();
        let timer = REFRESH_DURATION.start_timer();
        self.emit_snapshots(&cluster_id, snapshot, span);
        self.refresh_discovery(discovery.clone(), span)?;
        self.fetcher
            .fetch(discovery.clone(), lock.watch(), span)
            .with_context(|_| ErrorKind::ClusterRefresh)?;
        self.aggregator
            .aggregate(discovery, lock.watch(), span)
            .with_context(|_| ErrorKind::ClusterAggregation)?;

        // Done.
        timer.observe_duration();
        lock.release(span.context().clone())
            .context(ErrorKind::Coordination)?;
        info!(self.logger, "Cluster state refresh completed"; "cluster_id" => cluster_id);
        Ok(())
    }

    /// Emit cluster state snapshots, if needed by this task.
    fn emit_snapshots(&self, name: &str, snapshot: bool, span: &mut Span) {
        if !snapshot {
            return;
        }
        debug!(self.logger, "Emitting cluster snapshot"; "cluster" => name);
        span.log(Log::new().log("stage", "snapshot"));
        let snapshotter = Snapshotter::new(name.into(), self.events.clone(), self.store.clone());
        if let Err(error) = snapshotter.run(span) {
            capture_fail!(
                &error,
                self.logger,
                "Failed to emit snapshots";
                "cluster" => name,
                failure_info(&error),
            );
        }
    }

    /// Refresh the state of the cluster discovery.
    ///
    /// Refresh is performed based on the current state or luck of state.
    /// This method emits events as needed (and before the state is updated).
    fn refresh_discovery(&self, discovery: ClusterDiscovery, span: &mut Span) -> Result<()> {
        let current_state = self
            .store
            .cluster(discovery.cluster_id.clone())
            .discovery(span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStorePersist("cluster_discovery"))?;
        if let Some(current_state) = current_state {
            if discovery == current_state {
                return Ok(());
            }
            let event = Event::builder()
                .cluster()
                .changed(current_state, discovery.clone());
            let event_code = event.code();
            self.events
                .emit(event, span.context().clone())
                .with_context(|_| ErrorKind::EventsStreamEmit(event_code))?;
        } else {
            let event = Event::builder().cluster().cluster_new(discovery.clone());
            let event_code = event.code();
            self.events
                .emit(event, span.context().clone())
                .with_context(|_| ErrorKind::EventsStreamEmit(event_code))?;
        }
        self.store
            .persist()
            .cluster_discovery(discovery, span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStorePersist("cluster_discovery"))?;
        Ok(())
    }
}

impl TaskHandler<ReplicanteQueues> for Handler {
    fn handle(&self, task: Task) {
        let tracer = self.tracing.tracer();
        let mut span = tracer.span("tasks.cluster_refresh").auto_finish();
        // If the task is carring a tracing context set it as the parent span.
        match task.trace(&tracer) {
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
            .do_handle(&task, &mut span)
            .map_err(|error| fail_span(error, &mut span));
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
                    "Failed to handle cluster discovery task";
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

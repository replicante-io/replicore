//! Implmentation of the cluster state refresh process.
//!
//! Details of the refresh process are documented at
//!   https://www.replicante.io/docs/devnotes/main/notes/refresh/
use std::time::Duration;

use chrono::Utc;
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
use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::events::Event;
use replicante_models_core::scope::Namespace;
use replicante_service_coordinator::Coordinator;
use replicante_service_coordinator::ErrorKind as CoordinatorErrorKind;
use replicante_service_tasks::TaskHandler;
use replicante_store_primary::store::Store;
use replicante_stream_events::EmitMessage;
use replicante_stream_events::Stream as EventsStream;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_tracing::fail_span;

use super::payload::ClusterRefreshPayload;
use super::ReplicanteQueues;
use super::Task;
use crate::interfaces::tracing::Tracing;
use crate::Config;
use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

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

    // TODO: remove when namespaces are done properly from the primary store.
    tmp_global_namespace: Namespace,
}

impl Handler {
    pub fn new(
        config: &Config,
        interfaces: &Interfaces,
        logger: Logger,
        agents_timeout: Duration,
    ) -> Handler {
        let primary_store = interfaces.stores.primary.clone();
        let aggregator = Aggregator::new(logger.clone(), primary_store.clone());
        let coordinator = interfaces.coordinator.clone();
        let events = interfaces.streams.events.clone();
        let fetcher = Fetcher::new(
            logger.clone(),
            interfaces.streams.events.clone(),
            primary_store.clone(),
            agents_timeout,
            interfaces.tracing.tracer(),
        );
        let tracing = interfaces.tracing.clone();
        let tmp_global_namespace = config.tmp_namespace_settings.clone().into();
        Handler {
            aggregator,
            coordinator,
            events,
            fetcher,
            logger,
            store: primary_store,
            tracing,
            tmp_global_namespace,
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

        // Fetch cluster's namespace model.
        // TODO: replace with store access when namespaces are done properly.
        let ns = self.tmp_global_namespace.clone();

        // Refresh cluster state.
        let cluster_id = discovery.cluster_id.clone();
        let refresh_id = Utc::now().timestamp();
        let timer = REFRESH_DURATION.start_timer();
        self.emit_snapshots(&cluster_id, snapshot, span);
        self.refresh_discovery(discovery.clone(), span)?;
        self.fetcher
            .fetch(ns, discovery.clone(), refresh_id, lock.watch(), span)
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
            let code = event.code();
            let stream_key = event.stream_key();
            let event = EmitMessage::with(stream_key, event)
                .with_context(|_| ErrorKind::EventsStreamEmit(code))?
                .trace(span.context().clone());
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::EventsStreamEmit(code))?;
        } else {
            let event = Event::builder().cluster().new_cluster(discovery.clone());
            let code = event.code();
            let stream_key = event.stream_key();
            let event = EmitMessage::with(stream_key, event)
                .with_context(|_| ErrorKind::EventsStreamEmit(code))?
                .trace(span.context().clone());
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::EventsStreamEmit(code))?;
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

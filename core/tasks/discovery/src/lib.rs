//! Implmentation of the cluster records discovery process.
use std::sync::Arc;

use failure::ResultExt;
use opentracingrust::Span;
use opentracingrust::Tracer;
use slog::debug;
use slog::warn;
use slog::Logger;

use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::ClusterSettings;
use replicante_models_core::events::Event;
use replicante_service_tasks::TaskHandler;
use replicante_store_primary::store::Store;
use replicante_stream_events::EmitMessage;
use replicante_stream_events::Stream;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_tracing::fail_span;

use replicore_models_tasks::payload::DiscoverClustersPayload;
use replicore_models_tasks::ReplicanteQueues;
use replicore_models_tasks::Task;

mod error;
mod metrics;

pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::metrics::register_metrics;

use self::metrics::DISCOVER_CLUSTER_SETTINGS_COUNT;
use self::metrics::DISCOVER_DISABLED_COUNT;

/// Task handler for `ReplicanteQueues::DiscoverClusters` tasks.
pub struct DiscoverClusters {
    events: Stream,
    logger: Logger,
    store: Store,
    tracer: Arc<Tracer>,
}

impl DiscoverClusters {
    pub fn new(
        events: Stream,
        logger: Logger,
        store: Store,
        tracer: Arc<Tracer>,
    ) -> DiscoverClusters {
        DiscoverClusters {
            events,
            logger,
            store,
            tracer,
        }
    }

    fn handle_task(&self, task: &Task, span: &mut Span) -> Result<()> {
        let payload: DiscoverClustersPayload =
            task.deserialize().context(ErrorKind::DeserializePayload)?;
        span.tag("discovery.namespace", payload.settings.namespace.clone());
        span.tag("discovery.name", payload.settings.name.clone());
        span.tag("discovery.enabled", payload.settings.enabled);

        // Disabled discoveries should not make it to tasks but just in case they do skip them.
        if !payload.settings.enabled {
            DISCOVER_DISABLED_COUNT.inc();
            warn!(
                self.logger,
                "Skipping discovering clusters from disabled DiscoverySettings";
                "namespace" => payload.settings.namespace,
                "name" => payload.settings.name,
            );
            return Ok(());
        }

        let name = payload.settings.name.clone();
        let namespace = payload.settings.namespace.clone();
        let discoveries = replicante_cluster_discovery::discover(payload.settings);
        for record in discoveries {
            let record = record.with_context(|_| ErrorKind::fetch_cluster(&namespace, &name))?;
            debug!(
                self.logger,
                "Processing discovery record";
                "namespace" => &namespace,
                "name" => &record.cluster_id,
            );
            self.handle_record(&namespace, record, span)?;
        }
        Ok(())
    }

    fn handle_record(
        &self,
        namespace: &str,
        record: ClusterDiscovery,
        span: &mut Span,
    ) -> Result<()> {
        let cluster_id = record.cluster_id.clone();
        let namespace = namespace.to_string();
        let span_context = span.context().clone();

        // Fetch current record (if any) and emit "diff" events.
        let current_record = self
            .store
            .cluster(namespace.clone(), cluster_id.clone())
            .discovery(span_context.clone())
            .with_context(|_| ErrorKind::fetch_discovery(&namespace, &cluster_id))?;
        let event = match (current_record, &record) {
            (None, record) => Some(Event::builder().cluster().new_cluster(record.clone())),
            (Some(current), record) if current != *record => {
                Some(Event::builder().cluster().changed(current, record.clone()))
            }
            _ => None,
        };
        if let Some(event) = event {
            let code = event.code();
            let stream_key = event.entity_id().partition_key();
            let event = EmitMessage::with(stream_key, event)
                .with_context(|_| ErrorKind::emit_event(code))?
                .trace(span.context().clone());
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::emit_event(code))?;
        }

        // Persist the discovery record if it changed.
        self.store
            .persist()
            .cluster_discovery(record, span_context.clone())
            .with_context(|_| ErrorKind::persist_record(&namespace, &cluster_id))?;

        // Ensure a ClusterSettings record for the cluster exists.
        let settings = self
            .store
            .cluster(namespace.clone(), cluster_id.clone())
            .settings(span_context.clone())
            .with_context(|_| ErrorKind::fetch_settings(&namespace, &cluster_id))?;
        if settings.is_none() {
            DISCOVER_CLUSTER_SETTINGS_COUNT.inc();
            let settings = ClusterSettings::synthetic(&namespace, &cluster_id);
            let event = Event::builder()
                .cluster()
                .synthetic_settings(settings.clone());
            let code = event.code();
            let stream_key = event.entity_id().partition_key();
            let event = EmitMessage::with(stream_key, event)
                .with_context(|_| ErrorKind::emit_event(code))?
                .trace(span_context.clone());
            self.events
                .emit(event)
                .with_context(|_| ErrorKind::emit_event(code))?;
            self.store
                .persist()
                .cluster_settings(settings, span_context)
                .with_context(|_| ErrorKind::persist_settings(&namespace, &cluster_id))?;
        }
        Ok(())
    }
}

impl TaskHandler<ReplicanteQueues> for DiscoverClusters {
    fn handle(&self, task: Task) {
        let mut span = self.tracer.span("task.discover_clusters").auto_finish();
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

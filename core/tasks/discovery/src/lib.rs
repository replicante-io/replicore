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
use replicante_service_tasks::TaskHandler;
use replicante_store_primary::store::Store;
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
    logger: Logger,
    store: Store,
    tracer: Arc<Tracer>,
}

impl DiscoverClusters {
    pub fn new(logger: Logger, store: Store, tracer: Arc<Tracer>) -> DiscoverClusters {
        DiscoverClusters {
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

        let namespace = payload.settings.namespace.clone();
        let discoveries = replicante_cluster_discovery::discover(payload.settings);
        for record in discoveries {
            let record = record.context(ErrorKind::FetchCluster)?;
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

        // TODO: Fetch current record (if any).
        // TODO: "Diff" records and emit events.

        // Persist the discovery record if it changed.
        self.store
            .persist()
            .cluster_discovery(record, span_context.clone())
            .context(ErrorKind::PersistRecord)?;
        // Ensure a ClusterSettings record for the cluster exists.
        let settings = self
            .store
            .cluster(namespace.clone(), cluster_id.clone())
            .settings(span_context.clone())
            .context(ErrorKind::FetchSettings)?;
        if settings.is_none() {
            DISCOVER_CLUSTER_SETTINGS_COUNT.inc();
            let settings = ClusterSettings::new(namespace, cluster_id, true);
            self.store
                .persist()
                .cluster_settings(settings, span_context)
                .context(ErrorKind::PersistSettings)?;
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

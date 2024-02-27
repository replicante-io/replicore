use std::sync::Arc;

use failure::ResultExt;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use slog::debug;
use slog::Logger;

use replicante_models_core::cluster::discovery::DiscoverySettings;
use replicante_service_tasks::TaskRequest;
use replicante_store_primary::store::Store;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_tracing::fail_span;
use replicore_models_tasks::payload::DiscoverClustersPayload;
use replicore_models_tasks::ReplicanteQueues;
use replicore_models_tasks::Tasks;
use replisdk::core::models::platform::Platform;

use crate::metrics::SCHEDULE_COUNT;
use crate::ErrorKind;
use crate::Result;

/// Handle fetching and scheduling cluster discovery tasks.
pub struct DiscoveryLogic {
    logger: Logger,
    store: Store,
    tasks: Tasks,
    tracer: Arc<Tracer>,
}

impl DiscoveryLogic {
    pub fn new(logger: Logger, store: Store, tasks: Tasks, tracer: Arc<Tracer>) -> DiscoveryLogic {
        DiscoveryLogic {
            logger,
            store,
            tasks,
            tracer,
        }
    }

    /// Search for pending discovery tasks and schedule them.
    ///
    /// Update the next_run attribute when the discovery is scheduled.
    /// This prevents scheduling the same discovery repetitively in many situations:
    ///  * Slow or busy workers may fail to keep up (adding more work won't help).
    ///  * Incorrect configuration (short discovery loop intervals).
    ///  * One of many many possible bugs ...
    pub fn run(&self) -> Result<()> {
        let mut span = self
            .tracer
            .span("component.discover_clusters")
            .auto_finish();
        let span_context = span.context().clone();

        // Discover legacy DiscoverySettings.
        let discoveries = self
            .store
            .global_search()
            .discoveries_to_run(span_context.clone())
            .context(ErrorKind::DiscoveriesSearch)
            .map_err(|error| fail_span(error, &mut *span))?;
        for discovery in discoveries {
            self.schedule_discovery(discovery, span_context.clone())
                .map_err(|error| fail_span(error, &mut *span))?;
            SCHEDULE_COUNT.inc();
        }

        // Discover clusters from platforms.
        let platforms = self
            .store
            .global_search()
            .platform_discoveries(span_context.clone())
            .context(ErrorKind::DiscoveriesSearch)
            .map_err(|error| fail_span(error, &mut *span))?;
        for platform in platforms {
            self.schedule_platform(platform, span_context.clone())
                .map_err(|error| fail_span(error, &mut *span))?;
            SCHEDULE_COUNT.inc();
        }

        Ok(())
    }

    /// Process an individual DiscoverySettings record and schedule a discovery task for it.
    fn schedule_discovery(
        &self,
        discovery: replicante_store_primary::Result<DiscoverySettings>,
        span_context: SpanContext,
    ) -> Result<()> {
        let discovery = discovery.context(ErrorKind::DiscoveriesPartialSearch)?;
        debug!(
            self.logger,
            "Scheduling pending discovery";
            "namespace" => &discovery.namespace,
            "name" => &discovery.name,
        );

        // Enqueue clusters discovery task.
        let payload = DiscoverClustersPayload::new_discovery(discovery.clone());
        let mut task = TaskRequest::new(ReplicanteQueues::DiscoverClusters);
        if let Err(error) = task.trace(&span_context, &self.tracer) {
            let error = failure::SyncFailure::new(error);
            capture_fail!(
                &error,
                self.logger,
                "Unable to inject trace context in task request";
                "namespace" => &discovery.namespace,
                "name" => &discovery.name,
                failure_info(&error),
            );
        }
        if let Err(error) = self.tasks.request(task, payload) {
            capture_fail!(
                &error,
                self.logger,
                "Failed to request clusters discovery";
                "namespace" => &discovery.namespace,
                "name" => &discovery.name,
                failure_info(&error),
            );
        };

        // Update next_run attribute so we don't spam ourselves with tasks.
        let namespace = discovery.namespace.clone();
        let name = discovery.name.clone();
        self.store
            .persist()
            .next_discovery_run(discovery, span_context)
            .with_context(|_| ErrorKind::persist_next_run(namespace, name))?;
        Ok(())
    }

    /// Process an individual Platform record and schedule a discovery task for it.
    fn schedule_platform(
        &self,
        platform: replicante_store_primary::Result<Platform>,
        span_context: SpanContext,
    ) -> Result<()> {
        let platform = platform.context(ErrorKind::DiscoveriesPartialSearch)?;
        debug!(
            self.logger,
            "Scheduling pending platform discovery";
            "namespace" => &platform.ns_id,
            "name" => &platform.name,
        );

        // Enqueue clusters discovery task.
        let payload = DiscoverClustersPayload::new(platform.clone());
        let mut task = TaskRequest::new(ReplicanteQueues::DiscoverClusters);
        if let Err(error) = task.trace(&span_context, &self.tracer) {
            let error = failure::SyncFailure::new(error);
            capture_fail!(
                &error,
                self.logger,
                "Unable to inject trace context in task request";
                "namespace" => &platform.ns_id,
                "name" => &platform.name,
                failure_info(&error),
            );
        }
        if let Err(error) = self.tasks.request(task, payload) {
            capture_fail!(
                &error,
                self.logger,
                "Failed to request platform discovery";
                "namespace" => &platform.ns_id,
                "name" => &platform.name,
                failure_info(&error),
            );
        };

        // Update next_run attribute so we don't spam ourselves with tasks.
        let namespace = &platform.ns_id;
        let name = &platform.name;
        self.store
            .persist()
            .next_platform_discovery_run(&platform, span_context)
            .with_context(|_| ErrorKind::persist_next_run(namespace, name))?;
        Ok(())
    }
}

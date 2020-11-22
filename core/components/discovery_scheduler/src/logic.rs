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

use crate::metrics::DISCOVERY_SCHEDULE_COUNT;
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

    /// Searche for pending discovery tasks and schedule them.
    ///
    /// Update the next_run attribute when the discovery is scheduled.
    /// This prevents scheduling the same discovery ripetievly in many situations:
    ///  * Slow or busy workers may fail to keep up (adding more work won't help).
    ///  * Incorrect configuration (short discovery loop intervals).
    ///  * One of many many possible bugs ...
    pub fn run(&self) -> Result<()> {
        let mut span = self
            .tracer
            .span("component.discover_clusters")
            .auto_finish();
        let span_context = span.context().clone();
        let discoveries = self
            .store
            .global_search()
            .discoveries_to_run(span_context.clone())
            .context(ErrorKind::DiscoveriesSearch)
            .map_err(|error| fail_span(error, &mut *span))?;

        for discovery in discoveries {
            self.schedule_discovery(discovery, span_context.clone())
                .map_err(|error| fail_span(error, &mut *span))?;
            DISCOVERY_SCHEDULE_COUNT.inc();
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
        let payload = DiscoverClustersPayload::new(discovery.clone());
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
        self.store
            .persist()
            .next_discovery_run(discovery, span_context)
            .context(ErrorKind::DiscoveriesPartialSearch)?;
        Ok(())
    }
}

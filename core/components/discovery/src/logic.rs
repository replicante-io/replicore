use std::sync::Arc;

use failure::ResultExt;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use slog::debug;
use slog::Logger;

use replicante_models_core::cluster::discovery::DiscoverySettings;
use replicante_store_primary::store::Store;
use replicante_util_tracing::fail_span;

use crate::metrics::DISCOVERY_SCHEDULE_COUNT;
use crate::ErrorKind;
use crate::Result;

/// Handle fetching and scheduling cluster discovery tasks.
pub struct DiscoveryLogic {
    logger: Logger,
    store: Store,
    tracer: Arc<Tracer>,
}

impl DiscoveryLogic {
    pub fn new(logger: Logger, store: Store, tracer: Arc<Tracer>) -> DiscoveryLogic {
        DiscoveryLogic {
            logger,
            store,
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
        let mut span = self.tracer.span("discovery.schedule_pending").auto_finish();
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
            "name" => &discovery.name,
            "namespace" => &discovery.namespace,
        );
        // TODO: schedule discovery task.
        self.store
            .persist()
            .next_discovery_run(discovery, span_context)
            .context(ErrorKind::DiscoveriesPartialSearch)?;
        Ok(())
    }
}

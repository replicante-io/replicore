//use std::sync::Arc;

//use failure::ResultExt;
//use opentracingrust::SpanContext;
//use opentracingrust::Tracer;
//use slog::debug;
//use slog::Logger;

//use replicante_models_core::cluster::discovery::DiscoverySettings;
//use replicante_service_tasks::TaskRequest;
//use replicante_store_primary::store::Store;
//use replicante_util_failure::capture_fail;
//use replicante_util_failure::failure_info;
//use replicante_util_tracing::fail_span;

//use replicore_models_tasks::payload::DiscoverClustersPayload;
//use replicore_models_tasks::ReplicanteQueues;
//use replicore_models_tasks::Tasks;

//use crate::metrics::DISCOVERY_SCHEDULE_COUNT;
//use crate::ErrorKind;
use crate::Result;

/// Handle fetching and scheduling cluster discovery tasks.
pub struct Logic {
    //logger: Logger,
//    store: Store,
//    tasks: Tasks,
//    tracer: Arc<Tracer>,
}

impl Logic {
    //pub fn new(logger: Logger, store: Store, tasks: Tasks, tracer: Arc<Tracer>) -> Logic {
    pub fn new() -> Logic {
        Logic {
            //logger,
            //store,
            //tasks,
            //tracer,
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
        //let mut span = self
        //    .tracer
        //    .span("component.discover_clusters")
        //    .auto_finish();
        //let span_context = span.context().clone();
        //let discoveries = self
        //    .store
        //    .global_search()
        //    .discoveries_to_run(span_context.clone())
        //    .context(ErrorKind::DiscoveriesSearch)
        //    .map_err(|error| fail_span(error, &mut *span))?;

        //for discovery in discoveries {
        //    self.schedule_discovery(discovery, span_context.clone())
        //        .map_err(|error| fail_span(error, &mut *span))?;
        //    DISCOVERY_SCHEDULE_COUNT.inc();
        //}
        Ok(())
    }
}

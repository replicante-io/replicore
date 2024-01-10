use std::sync::Arc;
use std::time::Duration;

use failure::ResultExt;
use opentracingrust::Span;
use opentracingrust::Tracer;
use slog::info;
use slog::Logger;

use replicante_models_core::cluster::OrchestrateReportBuilder;
use replicante_service_coordinator::NonBlockingLock;
use replicante_store_primary::store::Store;
use replicante_stream_events::Stream;

use crate::metrics::SETTINGS_DISABLED_COUNT;
use crate::ErrorKind;
use crate::Result;

/// Orchestration logic split over multiple methods for ease of development.
pub struct Logic {
    agents_timeout: Duration,
    events: Stream,
    logger: Logger,
    store: Store,
    tracer: Arc<Tracer>,
}

impl Logic {
    pub fn new(
        agents_timeout: Duration,
        events: Stream,
        logger: Logger,
        store: Store,
        tracer: Arc<Tracer>,
    ) -> Logic {
        Logic {
            agents_timeout,
            events,
            logger,
            store,
            tracer,
        }
    }

    /// Orchestrate a cluster by syncing its state, scheduling or progressing actions, etc ...
    ///
    /// Architectural details of what the cluster orchestration process are documented
    /// in the developers notes section of the documentation.
    pub fn orchestrate<S1, S2>(
        &self,
        namespace: S1,
        cluster_id: S2,
        report: &mut OrchestrateReportBuilder,
        lock: &NonBlockingLock,
        span: &mut Span,
    ) -> Result<()>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let namespace = namespace.into();
        let cluster_id = cluster_id.into();
        let span_context = span.context().clone();

        // Fetch needed models from the primary store.
        //let namespace = ...;
        let settings = self
            .store
            .cluster(namespace.clone(), cluster_id.clone())
            .settings(span_context)
            .with_context(|_| ErrorKind::fetch_settings(&namespace, &cluster_id))?
            .ok_or_else(|| ErrorKind::settings_not_found(&namespace, &cluster_id))?;
        if !settings.enabled {
            SETTINGS_DISABLED_COUNT.inc();
            info!(
                self.logger,
                "Skipping orchestration of disabled ClusterSettings";
                "namespace" => &settings.namespace,
                "cluster_id" => &settings.cluster_id,
            );
            return Ok(());
        }

        // Perform cluster orchestration.
        let (data, data_mut) = replicore_cluster_orchestrate::init_data(
            settings,
            self.events.clone(),
            lock.watch(),
            self.logger.clone(),
            self.agents_timeout,
            report,
            self.store.clone(),
            Some(span),
            self.tracer.clone(),
        )
        .map_err(replicore_util_errors::AnyWrap::from)
        .with_context(|_| ErrorKind::orchestrate(&namespace, &cluster_id))?;
        replicore_cluster_orchestrate::orchestrate(&data, data_mut, &self.store)
            .map_err(replicore_util_errors::AnyWrap::from)
            .with_context(|_| ErrorKind::orchestrate(namespace, cluster_id))?;
        Ok(())
    }
}

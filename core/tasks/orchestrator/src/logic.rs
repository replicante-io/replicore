use std::sync::Arc;
use std::time::Duration;

use failure::ResultExt;
use opentracingrust::Span;
use opentracingrust::Tracer;
use slog::info;
use slog::warn;
use slog::Logger;

use replicante_cluster_aggregator::Aggregator;
use replicante_cluster_fetcher::Fetcher;
use replicante_models_core::cluster::ClusterSettings;
use replicante_models_core::scope::Namespace;
use replicante_service_coordinator::NonBlockingLock;
use replicante_store_primary::store::Store;
use replicante_stream_events::Stream;

use replicore_cluster_view::ClusterView;

use crate::metrics::SETTINGS_DISABLED_COUNT;
use crate::metrics::SYNC_DURATION;
use crate::ErrorKind;
use crate::Result;

/// Orchestration logic split over multiple methods for ease of development.
pub struct Logic {
    aggregator: Aggregator,
    fetcher: Fetcher,
    logger: Logger,
    store: Store,
}

impl Logic {
    pub fn new(
        agents_timeout: Duration,
        events: Stream,
        logger: Logger,
        store: Store,
        tracer: Arc<Tracer>,
    ) -> Logic {
        let aggregator = Aggregator::new(logger.clone(), store.clone());
        let fetcher = Fetcher::new(
            logger.clone(),
            events,
            store.clone(),
            agents_timeout,
            tracer,
        );
        Logic {
            aggregator,
            fetcher,
            logger,
            store,
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

        // Perform all orchestration steps.
        self.sync_cluster(settings, lock, span)?;
        Ok(())
    }

    /// Sync the state of each node in the cluster discovery record (if a record is found).
    fn sync_cluster(
        &self,
        settings: ClusterSettings,
        lock: &NonBlockingLock,
        span: &mut Span,
    ) -> Result<()> {
        let namespace_id = settings.namespace.clone();
        let cluster_id = settings.cluster_id.clone();
        let span_context = span.context().clone();
        let namespace = Namespace::HARDCODED_FOR_ROLLOUT();

        // Load the pre-refresh cluster view.
        // We can use the discovery record from this view as it will be the most recent
        // discovery stored in the DB regardless of the "freshness" of the other records.
        let cluster_view_before = self
            .store
            .cluster_view(namespace_id.clone(), cluster_id.clone(), span_context)
            .with_context(|_| {
                ErrorKind::build_cluster_view_from_store(&namespace_id, &cluster_id)
            })?;
        let mut cluster_view_after =
            ClusterView::builder(settings, cluster_view_before.discovery.clone())
                .map_err(crate::error::AnyWrap::from)
                .with_context(|_| {
                    ErrorKind::build_cluster_view_from_agents(&namespace_id, &cluster_id)
                })?;

        // Perform the sync process.
        info!(
            self.logger,
            "Sync state of cluster";
            "namespace" => &namespace_id,
            "cluster_id" => &cluster_id,
        );
        let timer = SYNC_DURATION.start_timer();
        self.fetcher
            .fetch(
                namespace,
                &cluster_view_before,
                &mut cluster_view_after,
                lock.watch(),
                span,
            )
            .with_context(|_| ErrorKind::refresh_cluster(&namespace_id, &cluster_id))?;
        if !lock.watch().inspect() {
            warn!(
                self.logger,
                "Cluster fetcher lock lost, skipping aggregation";
                "namespace" => &namespace_id,
                "cluster_id" => &cluster_id,
            );
            return Ok(());
        }
        let cluster_view_after = cluster_view_after.build();
        self.aggregator
            .aggregate(cluster_view_after, lock.watch(), span)
            .with_context(|_| ErrorKind::aggregate(namespace_id, cluster_id))?;
        timer.observe_duration();
        Ok(())
    }
}

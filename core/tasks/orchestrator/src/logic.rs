use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use failure::ResultExt;
use opentracingrust::Span;
use opentracingrust::Tracer;
use slog::debug;
use slog::info;
use slog::Logger;

use replicante_cluster_aggregator::Aggregator;
use replicante_cluster_fetcher::Fetcher;
use replicante_models_core::cluster::ClusterSettings;
use replicante_models_core::scope::Namespace;
use replicante_service_coordinator::NonBlockingLock;
use replicante_store_primary::store::Store;
use replicante_stream_events::Stream;

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
        let namespace_id = &settings.namespace;
        let cluster_id = &settings.cluster_id;
        let span_context = span.context().clone();
        let namespace = Namespace::HARDCODED_FOR_ROLLOUT();

        // Load the pre-refresh cluster view.
        // We can use the discovery record from this view as it will be the most recent
        // discovery stored in the DB regardless of the "freshness" of the other records.
        let cluster_view_before = self
            .store
            .cluster_view(
                namespace_id.to_string(),
                cluster_id.to_string(),
                span_context.clone(),
            )
            .with_context(|_| ErrorKind::build_cluster_view_from_store(namespace_id, cluster_id))?;
        // TODO: remove this as the aggregator is updated to use cluster views.
        let discovery = self
            .store
            .cluster(namespace_id.to_string(), cluster_id.to_string())
            .discovery(span_context)
            .with_context(|_| ErrorKind::fetch_discovery(namespace_id, cluster_id))?;
        let discovery = match discovery {
            Some(discovery) => discovery,
            None => {
                debug!(
                    self.logger,
                    "Skipping sync of cluster without discovery record";
                    "namespace" => &settings.namespace,
                    "cluster_id" => &settings.cluster_id,
                );
                return Ok(());
            }
        };

        // Perform the sync process.
        info!(
            self.logger,
            "Sync state of cluster";
            "namespace" => &settings.namespace,
            "cluster_id" => &settings.cluster_id,
        );
        let refresh_id = Utc::now().timestamp();
        let timer = SYNC_DURATION.start_timer();
        self.fetcher
            .fetch(
                namespace,
                &cluster_view_before,
                refresh_id,
                lock.watch(),
                span,
            )
            .with_context(|_| ErrorKind::refresh_cluster(namespace_id, cluster_id))?;
        self.aggregator
            .aggregate(discovery, lock.watch(), span)
            .with_context(|_| ErrorKind::aggregate(namespace_id, cluster_id))?;
        timer.observe_duration();
        Ok(())
    }
}

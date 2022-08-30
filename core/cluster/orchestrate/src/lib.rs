use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use opentracingrust::Span;
use opentracingrust::Tracer;
use slog::debug;
use slog::Logger;

use replicante_models_core::cluster::ClusterSettings;
use replicante_models_core::cluster::OrchestrateReportBuilder;
use replicante_models_core::cluster::SchedChoice;
use replicante_models_core::scope::Namespace;
use replicante_service_coordinator::NonBlockingLockWatcher;
use replicante_store_primary::store::Store;
use replicante_stream_events::Stream;
use replicore_cluster_view::ClusterView;
use replicore_cluster_view::ClusterViewBuilder;

pub mod errors;

mod aggregation;
mod metrics;
mod orchestrate_actions;
mod sched_choice;
mod sync;

#[cfg(test)]
mod tests;

pub use self::metrics::register_metrics;

/// Collect all immutable data for the cluster aggregation stage.
pub struct ClusterAggregateExtra {
    new_cluster_view: ClusterView,
}

/// Collect all mutable data for the cluster aggregation stage.
pub struct ClusterAggregateExtraMut<'cycle> {
    span: Option<&'cycle mut Span>,
}

/// Collect all immutable data for the cluster orchestration cycle.
pub struct ClusterOrchestrate {
    cluster_view: ClusterView,
    events: Stream,
    lock: NonBlockingLockWatcher,
    logger: Logger,
    // TODO(namespace-rollout): Drop this once a Namespace object is in ClusterView.
    namespace: Namespace,
    node_timeout: Duration,
    sched_choices: SchedChoice,
    store: Store,
    tracer: Arc<Tracer>,
}

/// Collect all mutable data for the cluster orchestration cycle.
pub struct ClusterOrchestrateMut<'cycle> {
    new_cluster_view: ClusterViewBuilder,
    report: &'cycle mut OrchestrateReportBuilder,
    span: Option<&'cycle mut Span>,
}

/// Initialise data objects for a cluster orchestration cycle.
#[allow(clippy::too_many_arguments)]
pub fn init_data<'cycle>(
    settings: ClusterSettings,
    events: Stream,
    lock: NonBlockingLockWatcher,
    logger: Logger,
    node_timeout: Duration,
    report: &'cycle mut OrchestrateReportBuilder,
    store: Store,
    span: Option<&'cycle mut Span>,
    tracer: Arc<Tracer>,
) -> Result<(ClusterOrchestrate, ClusterOrchestrateMut<'cycle>)> {
    let cluster_id = settings.cluster_id.clone();
    let namespace_id = settings.namespace.clone();
    let span_context = span.as_ref().map(|span| span.context().clone());

    // Load needed information from primary store.
    let namespace = Namespace::HARDCODED_FOR_ROLLOUT();
    let cluster_view = store
        .cluster_view(namespace_id.clone(), cluster_id.clone(), span_context)
        .map_err(failure::Fail::compat)
        .with_context(|| self::errors::InitError::cluster_view_load(&namespace_id, &cluster_id))?;
    let new_cluster_view = ClusterView::builder(settings, cluster_view.discovery.clone())
        .with_context(|| self::errors::InitError::cluster_view_init(&namespace_id, &cluster_id))?;

    // Derive additional attributes.
    let sched_choices = self::sched_choice::choose_scheduling(&cluster_view)
        .with_context(|| self::errors::InitError::scheduling_choice(&namespace_id, &cluster_id))?;
    report.action_scheduling_choices(sched_choices.clone());

    // Construct the object to handle the cycle.
    let data = ClusterOrchestrate {
        cluster_view,
        events,
        lock,
        logger,
        namespace,
        node_timeout,
        sched_choices,
        store,
        tracer,
    };
    let data_mut = ClusterOrchestrateMut {
        new_cluster_view,
        report,
        span,
    };
    Ok((data, data_mut))
}

/// Run through a cycle of cluster orchestration with the given parameters.
///
/// Docs on what the cluster orchestration steps are documented in the devnotes:
/// <https://www.replicante.io/docs/devnotes/main/notes/orchestration/>
pub fn orchestrate(data: &ClusterOrchestrate, mut data_mut: ClusterOrchestrateMut) -> Result<()> {
    debug!(
        data.logger,
        "Starting cluster orchestrate cycle";
        "namespace_id" => &data.cluster_view.namespace,
        "cluster_id" => &data.cluster_view.cluster_id,
    );

    // 1. Sync cluster state from nodes and schedule node actions.
    let timer = self::metrics::SYNC_DURATION.start_timer();
    self::sync::sync_cluster(data, &mut data_mut).map_err(|error| {
        self::metrics::SYNC_ERRORS_COUNT.inc();
        // TODO(open-telemetry): Proper error tagging on span.
        error
    })?;
    timer.observe_duration();

    // 2. Progress and schedule orchestration actions.
    let timer = self::metrics::ORCHESTRATE_ACTIONS_DURATION.start_timer();
    self::orchestrate_actions::orchestrate(data, &mut data_mut).map_err(|error| {
        self::metrics::ORCHESTRATE_ACTIONS_ERRORS_COUNT.inc();
        // TODO(open-telemetry): Proper error tagging on span.
        error
    })?;
    timer.observe_duration();

    // 3. Aggregate new cluster view into cluster as a whole data.
    let _timer = self::metrics::AGGREGATE_DURATION.start_timer();
    let extra = ClusterAggregateExtra {
        new_cluster_view: data_mut.new_cluster_view.build(),
    };
    let mut data_mut = ClusterAggregateExtraMut {
        span: data_mut.span,
    };
    self::aggregation::aggregate_cluster(data, &extra, &mut data_mut).map_err(|error| {
        self::metrics::AGGREGATE_ERRORS_COUNT.inc();
        // TODO(open-telemetry): Proper error tagging on span.
        error
    })

    // TODO: 4. Converge declarative clusters (by scheduling actions).
}

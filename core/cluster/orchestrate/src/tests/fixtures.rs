use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::ClusterSettings;
use replicante_models_core::cluster::OrchestrateReportBuilder;
use replicante_models_core::cluster::SchedChoice;
use replicante_models_core::scope::Namespace;
use replicante_service_coordinator::mock::MockCoordinator;
use replicante_service_coordinator::NonBlockingLock;
use replicante_store_primary::mock::Mock as MockStore;
use replicante_store_primary::store::Store;
use replicante_stream_events::Stream;
use replicore_cluster_view::ClusterView;
use replicore_cluster_view::ClusterViewBuilder;

use crate::ClusterOrchestrate;
use crate::ClusterOrchestrateMut;

pub const CLUSTER_ID: &str = "colours";
pub const NAMESPACE: &str = "default";

/// Collection of additional objects returned by the cluster fixture.
pub struct FixtureData {
    pub lock: NonBlockingLock,
    pub mock_store: MockStore,
}

/// Return cluster orchestration data to test orchestration functions.
pub fn cluster<'cycle, ClusterFill>(
    report: &'cycle mut OrchestrateReportBuilder,
    mut cluster_fill: ClusterFill,
) -> (
    ClusterOrchestrate,
    ClusterOrchestrateMut<'cycle>,
    FixtureData,
)
where
    ClusterFill: FnMut(&mut ClusterViewBuilder, &Store),
{
    // Create the cluster view.
    let discovery = ClusterDiscovery::new(
        CLUSTER_ID,
        vec![
            "node0".to_string(),
            "node1".to_string(),
            "node2".to_string(),
            "node3".to_string(),
        ],
    );
    let settings = ClusterSettings::synthetic(NAMESPACE, CLUSTER_ID);
    let mut cluster_view = ClusterView::builder(settings.clone(), discovery.clone())
        .expect("cluster view build should start");
    let new_cluster_view =
        ClusterView::builder(settings, discovery).expect("cluster view build should start");

    // Create "simpler" objects.
    let logger = slog::Logger::root(slog::Discard, slog::o!());
    let namespace = Namespace::HARDCODED_FOR_ROLLOUT();
    let sched_choices = SchedChoice::default();

    // Create mock interfaces.
    let events = Stream::mock();
    let mock_store = MockStore::default();
    let store = mock_store.store();
    let (tracer, _) = ::opentracingrust::tracers::NoopTracer::new();
    let mut lock = MockCoordinator::default()
        .mock()
        .non_blocking_lock("test.fixtures/core.cluster.orchestrate");
    lock.acquire(None).expect("lock to be acquired");

    // Fill cluster with fixture data.
    cluster_fill(&mut cluster_view, &store);

    // Package these arguments for orchestrate functions.
    let data = ClusterOrchestrate {
        cluster_view: cluster_view.build(),
        events,
        lock: lock.watch(),
        logger,
        namespace,
        node_timeout: std::time::Duration::from_millis(10),
        sched_choices,
        store,
        tracer: std::sync::Arc::new(tracer),
    };
    let data_mut = ClusterOrchestrateMut {
        new_cluster_view,
        report,
        span: None,
    };
    let fixture_data = FixtureData { lock, mock_store };
    (data, data_mut, fixture_data)
}

/// Instantiate the orchestrate report builder for tests to use.
pub fn orchestrate_report_builder() -> OrchestrateReportBuilder {
    let mut report = OrchestrateReportBuilder::new();
    report.for_cluster(NAMESPACE, CLUSTER_ID);
    report.start_now();
    report
}

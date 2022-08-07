use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use replicante_agent_client::mock::MockClient;
use replicante_agent_client::Client;
use replicante_agent_client::ErrorKind;
use replicante_models_agent::actions::ActionModel as RemoteAgentAction;
use replicante_models_agent::actions::ActionState as RemoteActionState;
use replicante_models_agent::info::AgentInfo as RemoteAgentInfo;
use replicante_models_agent::info::AgentVersion as RemoteAgentVersion;
use replicante_models_agent::info::DatastoreInfo as RemoteDatastoreInfo;
use replicante_models_agent::info::Shard as RemoteShard;
use replicante_models_agent::info::ShardRole;
use replicante_models_core::actions::node::Action;
use replicante_models_core::actions::node::ActionRequester;
use replicante_models_core::actions::node::ActionState;
use replicante_models_core::actions::node::ActionSyncSummary;
use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::AgentStatus;
use replicante_models_core::agent::Node;
use replicante_models_core::agent::Shard;
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

lazy_static::lazy_static! {
    pub static ref UUID1: Uuid = "a7514ce6-48f4-4f9d-bb22-78cbfc37c664".parse().unwrap();
    pub static ref UUID2: Uuid = "9084aec4-2234-4b9b-8a5d-aac914127255".parse().unwrap();
    pub static ref UUID3: Uuid = "be6ddf09-5c16-4be4-84dd-d03586eb1fc3".parse().unwrap();
    pub static ref UUID4: Uuid = "390ef9ab-ce0e-468e-977d-65873274c448".parse().unwrap();
    pub static ref UUID5: Uuid = "e5a023c6-78a3-4eb0-bc8f-6c5d057964ef".parse().unwrap();
    pub static ref UUID6: Uuid = "b9754ca6-824f-4796-8982-583888d2de19".parse().unwrap();
    pub static ref UUID7: Uuid = "141228ba-1651-11ed-861d-0242ac120002".parse().unwrap();
}

/// Collection of additional objects returned by the cluster fixture.
pub struct FixtureData {
    pub lock: NonBlockingLock,
    pub mock_store: MockStore,
}

/// Create an `AgentAction` returnable by a node client for tests.
pub fn agent_action(id: Uuid, finished: bool) -> RemoteAgentAction {
    let created_ts = Utc::now();
    let finished_ts = if finished { Some(Utc::now()) } else { None };
    let scheduled_ts = Utc::now();
    RemoteAgentAction {
        args: serde_json::json!({}),
        created_ts,
        finished_ts,
        headers: HashMap::new(),
        id,
        kind: "action".into(),
        requester: ActionRequester::AgentApi,
        scheduled_ts,
        state: if finished {
            RemoteActionState::Done
        } else {
            RemoteActionState::New
        },
        state_payload: None,
    }
}

/// Return cluster orchestration data to test orchestration functions.
pub fn cluster<'cycle>(
    report: &'cycle mut OrchestrateReportBuilder,
) -> (
    ClusterOrchestrate,
    ClusterOrchestrateMut<'cycle>,
    FixtureData,
) {
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

/// Convert a remote AgentAction into a core AgentAction for the test cluster.
pub fn core_action(node_id: &str, action: &RemoteAgentAction) -> Action {
    Action::new(CLUSTER_ID, node_id, action.clone())
}

/// Return a mock client that fails all requests.
pub fn mock_client_err() -> impl Client {
    MockClient::new(
        || Err(ErrorKind::Transport("test").into()),
        || Err(ErrorKind::Transport("test").into()),
        || Err(ErrorKind::Transport("test").into()),
    )
}

/// Return a mock client that handles all requests.
pub fn mock_client_ok() -> MockClient<
    impl Fn() -> replicante_agent_client::Result<RemoteAgentInfo>,
    impl Fn() -> replicante_agent_client::Result<RemoteDatastoreInfo>,
    impl Fn() -> replicante_agent_client::Result<replicante_models_agent::info::Shards>,
> {
    mock_client_with_node("node0")
}

/// Return a mock client with a given node ID that handles all requests.
pub fn mock_client_with_node(
    node_id: &str,
) -> MockClient<
    impl Fn() -> replicante_agent_client::Result<RemoteAgentInfo>,
    impl Fn() -> replicante_agent_client::Result<RemoteDatastoreInfo>,
    impl Fn() -> replicante_agent_client::Result<replicante_models_agent::info::Shards>,
> {
    let agent_info =
        RemoteAgentInfo::new(RemoteAgentVersion::new("abc", "1.2.3", "all the taints"));
    let node_info = RemoteDatastoreInfo::new(
        super::fixtures::CLUSTER_ID,
        "test.db",
        node_id,
        "1.2.3",
        None,
    );
    let shards = vec![
        RemoteShard::new("shard0", ShardRole::Secondary, None, None),
        RemoteShard::new("shard1", ShardRole::Unknown("test".into()), None, None),
    ];
    let shards = replicante_models_agent::info::Shards::new(shards);
    MockClient::new(
        move || Ok(agent_info.clone()),
        move || Ok(node_info.clone()),
        move || Ok(shards.clone()),
    )
}

/// Instantiate the orchestrate report builder for tests to use.
pub fn orchestrate_report_builder() -> OrchestrateReportBuilder {
    let mut report = OrchestrateReportBuilder::new();
    report.for_cluster(NAMESPACE, CLUSTER_ID);
    report.start_now();
    report
}

/// Fill a cluster with fixture data.
fn cluster_fill(cluster_view: &mut ClusterViewBuilder, store: &Store) {
    let agent = Agent {
        cluster_id: CLUSTER_ID.into(),
        host: "node0".into(),
        status: AgentStatus::NodeDown("fixture error".into()),
    };
    cluster_view.agent(agent.clone()).unwrap();
    store.persist().agent(agent, None).unwrap();

    let agent = AgentInfo {
        cluster_id: CLUSTER_ID.into(),
        host: "node0".into(),
        version_checkout: "def".into(),
        version_number: "4.5.6".into(),
        version_taint: "no taints".into(),
    };
    cluster_view.agent_info(agent.clone()).unwrap();
    store.persist().agent_info(agent, None).unwrap();

    let node = Node {
        cluster_display_name: None,
        cluster_id: CLUSTER_ID.into(),
        kind: "fixture.sql".into(),
        node_id: "node0".into(),
        version: "7.8.9".into(),
    };
    cluster_view.node(node.clone()).unwrap();
    store.persist().node(node, None).unwrap();

    let shard = Shard {
        cluster_id: CLUSTER_ID.into(),
        commit_offset: None,
        lag: None,
        node_id: "node0".into(),
        role: ShardRole::Primary,
        shard_id: "shard0".into(),
    };
    cluster_view.shard(shard.clone()).unwrap();
    store.persist().shard(shard, None).unwrap();

    let shard = Shard {
        cluster_id: CLUSTER_ID.into(),
        commit_offset: None,
        lag: None,
        node_id: "node0".into(),
        role: ShardRole::Secondary,
        shard_id: "shard1".into(),
    };
    cluster_view.shard(shard.clone()).unwrap();
    store.persist().shard(shard, None).unwrap();

    let action = agent_action(*UUID1, true);
    let action = core_action("node0", &action);
    store.persist().action(action, None).unwrap();

    let action = agent_action(*UUID2, true);
    let action = core_action("node0", &action);
    store.persist().action(action, None).unwrap();

    let action = agent_action(*UUID3, false);
    let action = core_action("node0", &action);
    let summary = ActionSyncSummary {
        cluster_id: CLUSTER_ID.into(),
        node_id: "node0".into(),
        action_id: action.action_id,
        state: action.state.clone(),
    };
    store.persist().action(action, None).unwrap();
    cluster_view.action(summary).unwrap();

    let action = agent_action(*UUID4, false);
    let action = core_action("node0", &action);
    let summary = ActionSyncSummary {
        cluster_id: CLUSTER_ID.into(),
        node_id: "node0".into(),
        action_id: action.action_id,
        state: action.state.clone(),
    };
    store.persist().action(action, None).unwrap();
    cluster_view.action(summary).unwrap();

    let action = agent_action(*UUID5, false);
    let mut action = core_action("node0", &action);
    action.state = ActionState::PendingApprove;
    let summary = ActionSyncSummary {
        cluster_id: CLUSTER_ID.into(),
        node_id: "node0".into(),
        action_id: action.action_id,
        state: action.state.clone(),
    };
    store.persist().action(action, None).unwrap();
    cluster_view.action(summary).unwrap();

    let action = agent_action(*UUID7, false);
    let mut action = core_action("node0", &action);
    action.state = ActionState::PendingSchedule;
    let summary = ActionSyncSummary {
        cluster_id: CLUSTER_ID.into(),
        node_id: "node0".into(),
        action_id: action.action_id,
        state: action.state.clone(),
    };
    store.persist().action(action, None).unwrap();
    cluster_view.action(summary).unwrap();
}

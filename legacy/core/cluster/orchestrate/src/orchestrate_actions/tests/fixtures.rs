use std::collections::HashMap;

use uuid::Uuid;

use replicante_models_core::actions::orchestrator::OrchestratorAction;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicante_models_core::actions::orchestrator::OrchestratorActionSyncSummary;
use replicante_models_core::cluster::OrchestrateReportBuilder;
use replicante_service_coordinator::NonBlockingLock;
use replicante_store_primary::mock::Mock as MockStore;
use replicante_store_primary::store::Store;
use replicore_cluster_view::ClusterViewBuilder;
use replicore_iface_orchestrator_action::OrchestratorActionRegistryBuilder;
use replicore_iface_orchestrator_action::TestRegistryClearGuard;

use crate::ClusterOrchestrate;
use crate::ClusterOrchestrateMut;

/// Collection of additional objects returned by the cluster fixture.
pub struct FixtureData {
    pub lock: NonBlockingLock,
    pub mock_store: MockStore,
    pub orchestrator_actions_registry_guard: TestRegistryClearGuard,
}

lazy_static::lazy_static! {
    pub static ref UUID1: Uuid = "a7514ce6-48f4-4f9d-bb22-78cbfc37c664".parse().unwrap();
    pub static ref UUID2: Uuid = "9084aec4-2234-4b9b-8a5d-aac914127255".parse().unwrap();
    pub static ref UUID3: Uuid = "be6ddf09-5c16-4be4-84dd-d03586eb1fc3".parse().unwrap();
    pub static ref UUID4: Uuid = "390ef9ab-ce0e-468e-977d-65873274c448".parse().unwrap();
    pub static ref UUID5: Uuid = "e5a023c6-78a3-4eb0-bc8f-6c5d057964ef".parse().unwrap();
    pub static ref UUID6: Uuid = "b9754ca6-824f-4796-8982-583888d2de19".parse().unwrap();
    pub static ref UUID7: Uuid = "141228ba-1651-11ed-861d-0242ac120002".parse().unwrap();
    pub static ref UUID8: Uuid = "696ac34c-2312-43f0-904d-02e21c4cf56d".parse().unwrap();
}

/// Return cluster orchestration data to test orchestration functions.
pub fn cluster<'cycle, ClusterFill>(
    report: &'cycle mut OrchestrateReportBuilder,
    cluster_fill: ClusterFill,
) -> (
    ClusterOrchestrate,
    ClusterOrchestrateMut<'cycle>,
    FixtureData,
)
where
    ClusterFill: Fn(&mut ClusterViewBuilder, &Store, &mut OrchestratorActionRegistryBuilder),
{
    let mut orchestrator_actions_builder = OrchestratorActionRegistryBuilder::empty();
    let (data, data_mut, fixture) = crate::tests::fixtures::cluster(report, |view, store| {
        cluster_fill(view, store, &mut orchestrator_actions_builder)
    });
    let orchestrator_actions_registry_guard = TestRegistryClearGuard::default();
    orchestrator_actions_builder.build_as_current();
    let new_fixture = FixtureData {
        lock: fixture.lock,
        mock_store: fixture.mock_store,
        orchestrator_actions_registry_guard,
    };
    (data, data_mut, new_fixture)
}

/// Fill the cluster with some pending orchestrate actions.
pub fn cluster_fill_pending_actions(
    cluster_view: &mut ClusterViewBuilder,
    store: &Store,
    orchestrator_actions_builder: &mut OrchestratorActionRegistryBuilder,
) {
    // Known Register orchestrator actions.
    replicore_action_debug::register(orchestrator_actions_builder).unwrap();

    // Fill cluster view and mock store.
    let args = serde_json::json!({"count": 5});
    let action = OrchestratorAction {
        cluster_id: crate::tests::fixtures::CLUSTER_ID.to_string(),
        action_id: *UUID6,
        args,
        created_ts: chrono::Utc::now(),
        finished_ts: None,
        headers: HashMap::new(),
        kind: "core.replicante.io/debug.counting".into(),
        scheduled_ts: None,
        state: OrchestratorActionState::PendingSchedule,
        state_payload: None,
        state_payload_error: None,
        timeout: None,
    };
    let summary = OrchestratorActionSyncSummary::from(&action);
    store.persist().orchestrator_action(action, None).unwrap();
    cluster_view.orchestrator_action(summary).unwrap();

    let args = serde_json::json!({"count": 5});
    let action = OrchestratorAction {
        cluster_id: crate::tests::fixtures::CLUSTER_ID.to_string(),
        action_id: *UUID7,
        args,
        created_ts: chrono::Utc::now(),
        finished_ts: None,
        headers: HashMap::new(),
        kind: "core.replicante.io/debug.counting".into(),
        scheduled_ts: None,
        state: OrchestratorActionState::PendingApprove,
        state_payload: None,
        state_payload_error: None,
        timeout: None,
    };
    let summary = OrchestratorActionSyncSummary::from(&action);
    store.persist().orchestrator_action(action, None).unwrap();
    cluster_view.orchestrator_action(summary).unwrap();

    let args = serde_json::json!({"count": 5});
    let action = OrchestratorAction {
        cluster_id: crate::tests::fixtures::CLUSTER_ID.to_string(),
        action_id: *UUID8,
        args,
        created_ts: chrono::Utc::now(),
        finished_ts: None,
        headers: HashMap::new(),
        kind: "core.replicante.io/debug.counting".into(),
        scheduled_ts: None,
        state: OrchestratorActionState::PendingSchedule,
        state_payload: None,
        state_payload_error: None,
        timeout: None,
    };
    let summary = OrchestratorActionSyncSummary::from(&action);
    store.persist().orchestrator_action(action, None).unwrap();
    cluster_view.orchestrator_action(summary).unwrap();
}

/// Fill the cluster with some running orchestrate actions.
pub fn cluster_fill_running_counts(
    cluster_view: &mut ClusterViewBuilder,
    store: &Store,
    orchestrator_actions_builder: &mut OrchestratorActionRegistryBuilder,
) {
    // Known Register orchestrator actions.
    replicore_action_debug::register(orchestrator_actions_builder).unwrap();

    // Fill cluster view and mock store.
    let args = serde_json::json!({"count": 5});
    let state_payload = serde_json::json!({"count_index": 2});
    let action = OrchestratorAction {
        cluster_id: crate::tests::fixtures::CLUSTER_ID.to_string(),
        action_id: *UUID1,
        args,
        created_ts: chrono::Utc::now(),
        finished_ts: None,
        headers: HashMap::new(),
        kind: "core.replicante.io/debug.counting".into(),
        scheduled_ts: None,
        state: OrchestratorActionState::Running,
        state_payload: state_payload.into(),
        state_payload_error: None,
        timeout: None,
    };
    let summary = OrchestratorActionSyncSummary::from(&action);
    store.persist().orchestrator_action(action, None).unwrap();
    cluster_view.orchestrator_action(summary).unwrap();

    let args = serde_json::json!({"count": 5});
    let state_payload = serde_json::json!({"count_index": 4});
    let action = OrchestratorAction {
        cluster_id: crate::tests::fixtures::CLUSTER_ID.to_string(),
        action_id: *UUID2,
        args,
        created_ts: chrono::Utc::now(),
        finished_ts: None,
        headers: HashMap::new(),
        kind: "core.replicante.io/debug.counting".into(),
        scheduled_ts: None,
        state: OrchestratorActionState::Running,
        state_payload: state_payload.into(),
        state_payload_error: None,
        timeout: None,
    };
    let summary = OrchestratorActionSyncSummary::from(&action);
    store.persist().orchestrator_action(action, None).unwrap();
    cluster_view.orchestrator_action(summary).unwrap();
}

pub fn cluster_fill_running_fail(
    cluster_view: &mut ClusterViewBuilder,
    store: &Store,
    orchestrator_actions_builder: &mut OrchestratorActionRegistryBuilder,
) {
    // Known Register orchestrator actions.
    replicore_action_debug::register(orchestrator_actions_builder).unwrap();

    // Fill cluster view and mock store.
    let args = serde_json::json!(null);
    let action = OrchestratorAction {
        cluster_id: crate::tests::fixtures::CLUSTER_ID.to_string(),
        action_id: *UUID3,
        args,
        created_ts: chrono::Utc::now(),
        finished_ts: None,
        headers: HashMap::new(),
        kind: "core.replicante.io/debug.fail".into(),
        scheduled_ts: None,
        state: OrchestratorActionState::Running,
        state_payload: None,
        state_payload_error: None,
        timeout: None,
    };
    let summary = OrchestratorActionSyncSummary::from(&action);
    store.persist().orchestrator_action(action, None).unwrap();
    cluster_view.orchestrator_action(summary).unwrap();
}

pub fn cluster_fill_running_missing(
    cluster_view: &mut ClusterViewBuilder,
    store: &Store,
    _: &mut OrchestratorActionRegistryBuilder,
) {
    // Fill cluster view and mock store.
    let args = serde_json::json!(null);
    let action = OrchestratorAction {
        cluster_id: crate::tests::fixtures::CLUSTER_ID.to_string(),
        action_id: *UUID4,
        args,
        created_ts: chrono::Utc::now(),
        finished_ts: None,
        headers: HashMap::new(),
        kind: "test.core.replicante.io/action.not.found".into(),
        scheduled_ts: None,
        state: OrchestratorActionState::Running,
        state_payload: None,
        state_payload_error: None,
        timeout: None,
    };
    let summary = OrchestratorActionSyncSummary::from(&action);
    store.persist().orchestrator_action(action, None).unwrap();
    cluster_view.orchestrator_action(summary).unwrap();
}

pub fn cluster_fill_running_success(
    cluster_view: &mut ClusterViewBuilder,
    store: &Store,
    orchestrator_actions_builder: &mut OrchestratorActionRegistryBuilder,
) {
    // Known Register orchestrator actions but ignore errors if we are already registered.
    let _ = replicore_action_debug::register(orchestrator_actions_builder);

    // Fill cluster view and mock store.
    let args = serde_json::json!(null);
    let action = OrchestratorAction {
        cluster_id: crate::tests::fixtures::CLUSTER_ID.to_string(),
        action_id: *UUID5,
        args,
        created_ts: chrono::Utc::now(),
        finished_ts: None,
        headers: HashMap::new(),
        kind: "core.replicante.io/debug.success".into(),
        scheduled_ts: None,
        state: OrchestratorActionState::Running,
        state_payload: None,
        state_payload_error: None,
        timeout: None,
    };
    let summary = OrchestratorActionSyncSummary::from(&action);
    store.persist().orchestrator_action(action, None).unwrap();
    cluster_view.orchestrator_action(summary).unwrap();
}

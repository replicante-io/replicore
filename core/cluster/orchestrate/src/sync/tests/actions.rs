use replicante_models_agent::actions::api::ActionInfoResponse;
use replicante_models_agent::actions::ActionListItem;
use replicante_models_agent::actions::ActionState as RemoteActionState;
use replicante_models_core::actions::node::ActionState;
use replicante_models_core::actions::node::ActionSyncSummary;

use super::fixtures::UUID1;
use super::fixtures::UUID2;
use super::fixtures::UUID3;
use super::fixtures::UUID4;
use super::fixtures::UUID5;
use super::fixtures::UUID6;
use super::fixtures::UUID7;

#[test]
fn fetch_remote_ids() {
    let mut client = super::fixtures::mock_client_ok();
    client.actions_queue = vec![ActionListItem {
        id: *UUID1,
        kind: "test".into(),
        state: RemoteActionState::New,
    }];
    client.actions_finished = vec![
        ActionListItem {
            id: *UUID2,
            kind: "test".into(),
            state: RemoteActionState::Done,
        },
        ActionListItem {
            id: *UUID3,
            kind: "test".into(),
            state: RemoteActionState::Done,
        },
    ];

    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, _fixture) = super::fixtures::cluster(&mut report);
    let ids =
        crate::sync::actions::fetch_remote_actions_ids(&data, &mut data_mut, &client, "node0")
            .expect("failed to fetch remote ids");
    assert_eq!(ids, vec![*UUID1, *UUID2, *UUID3]);
}

// This test cover the case of actions being finished between
// the call to /finish and the call to /queue.
#[test]
fn fetch_remote_ids_with_duplicates() {
    let mut client = super::fixtures::mock_client_ok();
    client.actions_queue = vec![
        ActionListItem {
            id: *UUID3,
            kind: "test".into(),
            state: RemoteActionState::Running,
        },
        ActionListItem {
            id: *UUID2,
            kind: "test".into(),
            state: RemoteActionState::New,
        },
        ActionListItem {
            id: *UUID1,
            kind: "test".into(),
            state: RemoteActionState::New,
        },
    ];
    client.actions_finished = vec![
        ActionListItem {
            id: *UUID2,
            kind: "test".into(),
            state: RemoteActionState::Done,
        },
        ActionListItem {
            id: *UUID3,
            kind: "test".into(),
            state: RemoteActionState::Done,
        },
        ActionListItem {
            id: *UUID4,
            kind: "test".into(),
            state: RemoteActionState::Done,
        },
    ];

    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, _fixture) = super::fixtures::cluster(&mut report);
    let ids =
        crate::sync::actions::fetch_remote_actions_ids(&data, &mut data_mut, &client, "node0")
            .expect("failed to fetch remote ids");
    assert_eq!(ids, vec![*UUID3, *UUID2, *UUID1, *UUID4]);
}

#[test]
fn sync_action_new() {
    let mut client = super::fixtures::mock_client_ok();
    let mut action = super::fixtures::agent_action(*UUID6, false);
    action.state = RemoteActionState::New;
    let info = ActionInfoResponse {
        action: action.clone(),
        history: Vec::new(),
    };
    client.actions.insert(*UUID6, info);

    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) = super::fixtures::cluster(&mut report);
    crate::sync::actions::sync_agent_action(&data, &mut data_mut, &client, "node0", *UUID6)
        .expect("action sync failed");

    let key = (
        crate::tests::fixtures::CLUSTER_ID.into(),
        "node0".into(),
        *UUID6,
    );
    let action = fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned")
        .actions
        .get(&key)
        .cloned()
        .expect("expected action not found");
    assert_eq!(action.action_id, *UUID6);
    assert_eq!(action.state, ActionState::New);
}

#[test]
fn sync_action_update() {
    let mut client = super::fixtures::mock_client_ok();
    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, fixture) = super::fixtures::cluster(&mut report);

    // Create an action in the store.
    let mut action = super::fixtures::agent_action(*UUID3, false);
    action.state = RemoteActionState::Running;
    let info = ActionInfoResponse {
        action: action.clone(),
        history: Vec::new(),
    };
    client.actions.insert(*UUID3, info);

    crate::sync::actions::sync_agent_action(&data, &mut data_mut, &client, "node0", *UUID3)
        .expect("action sync failed");

    let key = (
        crate::tests::fixtures::CLUSTER_ID.into(),
        "node0".into(),
        *UUID3,
    );
    let action = fixture
        .mock_store
        .state
        .lock()
        .expect("MockState lock poisoned")
        .actions
        .get(&key)
        .cloned()
        .expect("expected action not found");
    assert_eq!(action.action_id, *UUID3);
    assert_eq!(action.state, ActionState::Running);
}

#[test]
fn sync_lost_actions() {
    let client = super::fixtures::mock_client_ok();
    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, _fixture) = super::fixtures::cluster(&mut report);

    crate::sync::actions::sync_node_actions(&data, &mut data_mut, &client, "node0")
        .expect("actions sync failed");

    let action = data
        .store
        .action(crate::tests::fixtures::CLUSTER_ID.to_string(), *UUID1)
        .get(None)
        .expect("action to be in store")
        .expect("action to be in store");
    assert_eq!(action.state, ActionState::Done);
    let action = data
        .store
        .action(crate::tests::fixtures::CLUSTER_ID.to_string(), *UUID2)
        .get(None)
        .expect("action to be in store")
        .expect("action to be in store");
    assert_eq!(action.state, ActionState::Done);
    let action = data
        .store
        .action(crate::tests::fixtures::CLUSTER_ID.to_string(), *UUID3)
        .get(None)
        .expect("action to be in store")
        .expect("action to be in store");
    assert_eq!(action.state, ActionState::Lost);
    let action = data
        .store
        .action(crate::tests::fixtures::CLUSTER_ID.to_string(), *UUID4)
        .get(None)
        .expect("action to be in store")
        .expect("action to be in store");
    assert_eq!(action.state, ActionState::Lost);
}

#[test]
fn sync_schedule_pending_actions() {
    let client = super::fixtures::mock_client_ok();
    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, _fixture) = super::fixtures::cluster(&mut report);

    crate::sync::actions::sync_node_actions(&data, &mut data_mut, &client, "node0")
        .expect("actions sync failed");

    // Check request sent to the agent.
    let actions_to_schedule = client
        .actions_to_schedule
        .lock()
        .expect("agent MockClient::actions_to_schedule lock poisoned");
    assert_eq!(actions_to_schedule.len(), 1);
    let (kind, request) = actions_to_schedule
        .get(0)
        .expect("schedule action request")
        .clone();
    assert_eq!(request.action_id, Some(*UUID7));
    assert_eq!(kind, "action");
}

#[test]
fn sync_schedule_pending_actions_blocked() {
    let client = super::fixtures::mock_client_ok();
    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (mut data, mut data_mut, _fixture) = super::fixtures::cluster(&mut report);
    data.sched_choices.block_node = true;

    crate::sync::actions::sync_node_actions(&data, &mut data_mut, &client, "node0")
        .expect("actions sync failed");

    // Check request sent to the agent.
    let actions_to_schedule = client
        .actions_to_schedule
        .lock()
        .expect("agent MockClient::actions_to_schedule lock poisoned");
    assert_eq!(actions_to_schedule.len(), 0);
}

#[test]
fn track_unfinished_actions_in_cluster_view() {
    let mut client = super::fixtures::mock_client_ok();
    let mut report = crate::tests::fixtures::orchestrate_report_builder();
    let (data, mut data_mut, _fixture) = super::fixtures::cluster(&mut report);

    let action = super::fixtures::agent_action(*UUID3, true);
    client.actions_finished.push(ActionListItem {
        id: *UUID3,
        kind: action.kind.clone(),
        state: action.state.clone(),
    });
    let info = ActionInfoResponse {
        action,
        history: Vec::new(),
    };
    client.actions.insert(*UUID3, info);
    let mut action = super::fixtures::agent_action(*UUID4, false);
    action.state = RemoteActionState::Running;
    client.actions_queue.push(ActionListItem {
        id: *UUID4,
        kind: action.kind.clone(),
        state: action.state.clone(),
    });
    let info = ActionInfoResponse {
        action,
        history: Vec::new(),
    };
    client.actions.insert(*UUID4, info);

    crate::sync::actions::sync_node_actions(&data, &mut data_mut, &client, "node0")
        .expect("actions sync failed");

    // Finish new cluster view and check actions in it.
    let new_cluster_view = data_mut.new_cluster_view.build();
    assert_eq!(new_cluster_view.actions_unfinished_by_node.len(), 1);
    let actions = new_cluster_view
        .actions_unfinished_by_node
        .get("node0")
        .expect("missing actions for expected node");
    assert_eq!(
        actions[0],
        ActionSyncSummary {
            cluster_id: crate::tests::fixtures::CLUSTER_ID.into(),
            node_id: "node0".into(),
            action_id: *UUID4,
            state: ActionState::Running,
        },
    );
    assert_eq!(
        actions[1],
        ActionSyncSummary {
            cluster_id: crate::tests::fixtures::CLUSTER_ID.into(),
            node_id: "node0".into(),
            action_id: *UUID5,
            state: ActionState::PendingApprove,
        },
    );
    assert_eq!(
        actions[2],
        ActionSyncSummary {
            cluster_id: crate::tests::fixtures::CLUSTER_ID.into(),
            node_id: "node0".into(),
            action_id: *UUID7,
            state: ActionState::PendingSchedule,
        },
    );
}

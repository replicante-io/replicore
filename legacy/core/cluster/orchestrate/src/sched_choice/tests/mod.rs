mod fixtures;

use replicante_models_core::actions::node::ActionSyncSummary;
use replicante_models_core::actions::orchestrator::OrchestratorActionSyncSummary;
use replicante_models_core::cluster::SchedChoice;
use replicante_models_core::cluster::SchedChoiceReason;
use replicore_cluster_view::ClusterView;

use super::choose_scheduling;

/// Build a ClusterView with the given node and orchestrator action information.
fn build_view(
    node: Vec<ActionSyncSummary>,
    orchestrator: Vec<OrchestratorActionSyncSummary>,
) -> ClusterView {
    let mut view = self::fixtures::start_view_builder();
    for action in node {
        view.action(action).expect("cluster view to update");
    }
    for action in orchestrator {
        view.orchestrator_action(action)
            .expect("cluster view to update");
    }
    view.build()
}

/// Standard body for scheduling choice tests.
///
/// Scheduling choice tests revolve around selecting which actions are pending/running and
/// ensuring that the choice made is the expected one.
///
/// This involves a large amount of repetition when running the test so this function
/// exists to implement as much of the common logic as possible for all tests.
fn choose_scheduling_test(
    node_actions: Vec<ActionSyncSummary>,
    orchestrator_actions: Vec<OrchestratorActionSyncSummary>,
    expected_choice: SchedChoice,
) -> SchedChoice {
    let cluster = build_view(node_actions, orchestrator_actions);
    let _reg = self::fixtures::orchestrator_actions_registry();
    let actual_choice = choose_scheduling(&cluster).expect("choice to be made successfully");
    assert_eq!(expected_choice, actual_choice);
    actual_choice
}

#[test]
fn node_pending_alone() {
    let mut expected = SchedChoice::default();
    expected.block_orchestrator_exclusive = true;
    expected.reasons.push(SchedChoiceReason::FoundNodePending);
    let node_actions = vec![self::fixtures::node_action_pending()];
    let orchestrator_actions = vec![];
    choose_scheduling_test(node_actions, orchestrator_actions, expected);
}

#[test]
fn node_pending_with_orchestrator_exclusive_pending() {
    let mut expected = SchedChoice::default();
    expected.block_orchestrator_exclusive = true;
    expected.reasons.push(SchedChoiceReason::FoundNodeRunning);
    expected
        .reasons
        .push(SchedChoiceReason::FoundOrchestratorExclusivePending);
    let node_actions = vec![self::fixtures::node_action_running()];
    let orchestrator_actions = vec![self::fixtures::orchestrator_action_exclusive_pending()];
    choose_scheduling_test(node_actions, orchestrator_actions, expected);
}

#[test]
fn node_running_alone() {
    let mut expected = SchedChoice::default();
    expected.block_orchestrator_exclusive = true;
    expected.reasons.push(SchedChoiceReason::FoundNodeRunning);
    let node_actions = vec![self::fixtures::node_action_running()];
    let orchestrator_actions = vec![];
    choose_scheduling_test(node_actions, orchestrator_actions, expected);
}

#[test]
fn node_running_with_orchestrator_exclusive_pending() {
    let mut expected = SchedChoice::default();
    expected.block_orchestrator_exclusive = true;
    expected.reasons.push(SchedChoiceReason::FoundNodeRunning);
    expected
        .reasons
        .push(SchedChoiceReason::FoundOrchestratorExclusivePending);
    let node_actions = vec![self::fixtures::node_action_running()];
    let orchestrator_actions = vec![self::fixtures::orchestrator_action_exclusive_pending()];
    choose_scheduling_test(node_actions, orchestrator_actions, expected);
}

#[test]
fn node_running_with_orchestrator_exclusive_running() {
    let mut expected = SchedChoice::default();
    expected.block_node = true;
    expected.block_orchestrator_exclusive = true;
    expected.reasons.push(SchedChoiceReason::FoundNodeRunning);
    expected
        .reasons
        .push(SchedChoiceReason::FoundOrchestratorExclusiveRunning);
    let node_actions = vec![self::fixtures::node_action_running()];
    let orchestrator_actions = vec![self::fixtures::orchestrator_action_exclusive_running()];
    choose_scheduling_test(node_actions, orchestrator_actions, expected);
}

#[test]
fn node_running_with_node_pending() {
    let mut expected = SchedChoice::default();
    expected.block_orchestrator_exclusive = true;
    expected.reasons.push(SchedChoiceReason::FoundNodePending);
    expected.reasons.push(SchedChoiceReason::FoundNodeRunning);
    let node_actions = vec![
        self::fixtures::node_action_running(),
        self::fixtures::node_action_pending(),
    ];
    let orchestrator_actions = vec![];
    choose_scheduling_test(node_actions, orchestrator_actions, expected);
}

#[test]
fn orchestrator_exclusive_pending_alone() {
    let mut expected = SchedChoice::default();
    expected
        .reasons
        .push(SchedChoiceReason::FoundOrchestratorExclusivePending);
    let node_actions = vec![];
    let orchestrator_actions = vec![self::fixtures::orchestrator_action_exclusive_pending()];
    choose_scheduling_test(node_actions, orchestrator_actions, expected);
}

#[test]
fn orchestrator_exclusive_running_alone() {
    let mut expected = SchedChoice::default();
    expected.block_node = true;
    expected.block_orchestrator_exclusive = true;
    expected
        .reasons
        .push(SchedChoiceReason::FoundOrchestratorExclusiveRunning);
    let node_actions = vec![];
    let orchestrator_actions = vec![self::fixtures::orchestrator_action_exclusive_running()];
    choose_scheduling_test(node_actions, orchestrator_actions, expected);
}

#[test]
fn orchestrator_exclusive_running_with_node_pending() {
    let mut expected = SchedChoice::default();
    expected.block_node = true;
    expected.block_orchestrator_exclusive = true;
    expected.reasons.push(SchedChoiceReason::FoundNodePending);
    expected
        .reasons
        .push(SchedChoiceReason::FoundOrchestratorExclusiveRunning);
    let node_actions = vec![self::fixtures::node_action_pending()];
    let orchestrator_actions = vec![self::fixtures::orchestrator_action_exclusive_running()];
    choose_scheduling_test(node_actions, orchestrator_actions, expected);
}

#[test]
fn orchestrator_exclusive_running_with_orchestrator_exclusive_pending() {
    let mut expected = SchedChoice::default();
    expected.block_node = true;
    expected.block_orchestrator_exclusive = true;
    expected
        .reasons
        .push(SchedChoiceReason::FoundOrchestratorExclusivePending);
    expected
        .reasons
        .push(SchedChoiceReason::FoundOrchestratorExclusiveRunning);
    let node_actions = vec![];
    let orchestrator_actions = vec![
        self::fixtures::orchestrator_action_exclusive_running(),
        self::fixtures::orchestrator_action_exclusive_pending(),
    ];
    choose_scheduling_test(node_actions, orchestrator_actions, expected);
}

use anyhow::Result;

use replicante_models_core::actions::orchestrator::OrchestratorActionScheduleMode;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicante_models_core::cluster::SchedChoice;
use replicante_models_core::cluster::SchedChoiceReason;
use replicore_cluster_view::ClusterView;
use replicore_iface_orchestrator_action::OrchestratorActionRegistry;

use crate::SchedChoiceError;

#[cfg(test)]
mod tests;

/// Inspect a cluster view to make scheduling choices.
///
/// ## Improving performance of action scheduling
///
/// This function as written expects a full cluster view but really only needs the
/// action sync records and cluster identifiers to make its choices.
/// By requiring a full cluster view object we can only make our decision based on the
/// known state BEFORE the cluster state refresh: actions may have finished we consider running.
///
/// While generating a new cluster view before actions are scheduled just for this would be
/// inefficient it SHOULD be possible to rework this function to take directly what is needed
/// instead of the full cluster view.
/// The sync logic could then call us with action sync information derived from the combined
/// initial view and cluster state refresh.
///
/// Improving this function to use the latest available data SHOULD lead to more efficient
/// action scheduling as sync cycles are not wasted between an action being safe to schedule
/// and this logic actually knowing about that.
pub fn choose_scheduling(cluster: &ClusterView) -> Result<SchedChoice> {
    let mut any_node_pending = false;
    let mut any_node_running = false;
    let mut any_orchestrator_exclusive_pending = false;
    let mut any_orchestrator_exclusive_running = false;

    // Inspect node actions.
    for action in cluster.actions_unfinished_by_node.values().flatten() {
        any_node_pending |= action.state.is_pending_schedule();
        any_node_running |= action.state.is_running();
    }

    // Inspect orchestration actions.
    let registry = OrchestratorActionRegistry::current();
    for action in &cluster.actions_unfinished_orchestrator {
        let mode = registry
            .lookup(&action.kind)
            .ok_or_else(|| {
                SchedChoiceError::orchestrator_action_not_found(
                    &action.kind,
                    &cluster.namespace,
                    &cluster.cluster_id,
                )
            })?
            .metadata
            .schedule_mode;
        let is_exclusive = matches!(mode, OrchestratorActionScheduleMode::Exclusive);
        let is_pending = matches!(action.state, OrchestratorActionState::PendingSchedule);
        let is_running = action.state.is_running();

        // Track information about the action to update the choice at the end.
        any_orchestrator_exclusive_pending |= is_pending && is_exclusive;
        any_orchestrator_exclusive_running |= is_running && is_exclusive;
    }

    // Make the scheduling choice.
    let mut choice = SchedChoice {
        block_node: any_orchestrator_exclusive_running,
        block_orchestrator_exclusive: any_node_pending
            || any_node_running
            || any_orchestrator_exclusive_running,
        ..SchedChoice::default()
    };

    // Add reasons for the choice.
    if any_node_pending {
        choice.reasons.push(SchedChoiceReason::FoundNodePending);
    }
    if any_node_running {
        choice.reasons.push(SchedChoiceReason::FoundNodeRunning);
    }
    if any_orchestrator_exclusive_pending {
        choice
            .reasons
            .push(SchedChoiceReason::FoundOrchestratorExclusivePending);
    }
    if any_orchestrator_exclusive_running {
        choice
            .reasons
            .push(SchedChoiceReason::FoundOrchestratorExclusiveRunning);
    }
    Ok(choice)
}

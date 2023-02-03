use std::collections::HashSet;

use anyhow::Result;
use uuid::Uuid;

use replicante_models_core::actions::orchestrator::OrchestratorActionScheduleMode;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicante_store_primary::store::Store;
use replicore_iface_orchestrator_action::OrchestratorActionRegistry;

use crate::ClusterOrchestrate;
use crate::ClusterOrchestrateMut;

/// Start a pending orchestration action.
pub fn start_action(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    action_id: Uuid,
    exclusive_by_mode: &mut HashSet<OrchestratorActionScheduleMode>,
    store: &Store,
) -> Result<()> {
    // Get the action and the action implementation handler and metadata.
    let action_record = super::utils::get_orchestrator_action(data, data_mut, action_id)?;
    let registry = OrchestratorActionRegistry::current();
    let action = registry.lookup(&action_record.kind);
    let action = match action {
        None => {
            let error = anyhow::anyhow!(crate::errors::ActionError::unknown_kind(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                action_id,
                &action_record.kind,
            ));
            return super::utils::fail_action(data, data_mut, action_record, error);
        }
        Some(action) => action,
    };
    if data
        .sched_choices
        .is_mode_blocked(&action.metadata.schedule_mode)
    {
        return Ok(());
    }

    // Skip starting the action if it is exclusive with another running action.
    let is_exclusive = action.metadata.schedule_mode.is_exclusive();
    let is_exclusive_with_mode = action.metadata.schedule_mode.is_exclusive_with_mode();
    let skip = (is_exclusive && !exclusive_by_mode.is_empty())
        || (is_exclusive_with_mode && exclusive_by_mode.contains(&action.metadata.schedule_mode));
    if skip {
        slog::debug!(
            data.logger,
            "Skipped scheduling PendingSchedule orchestrator action due to exclusivity clash";
            "namespace" => &data.cluster_view.namespace,
            "cluster_id" => &data.cluster_view.cluster_id,
            "action_id" => action_id.to_string(),
        );
        return Ok(());
    }

    // Call shared start/progress action logic.
    let state_after = super::progress::run_action(data, data_mut, action_record, action, store)?;

    // If the action is still PendingSchedule it is poorly implemented. Fail it.
    if matches!(state_after, OrchestratorActionState::PendingSchedule) {
        // We need to reload the potentially changed action from the DB.
        let action_record = super::utils::get_orchestrator_action(data, data_mut, action_id)?;
        let error = anyhow::anyhow!(crate::errors::ActionError::did_not_start(
            &data.namespace.ns_id,
            &data.cluster_view.cluster_id,
            action_id,
            &action_record.kind,
        ));
        return super::utils::fail_action(data, data_mut, action_record, error);
    }

    // Track exclusive running actions so we don't start more then one.
    if state_after.is_running()
        && action.metadata.schedule_mode.is_exclusive()
        && action.metadata.schedule_mode.is_exclusive_with_mode()
    {
        exclusive_by_mode.insert(action.metadata.schedule_mode);
    }

    Ok(())
}

use anyhow::Context;
use anyhow::Result;

use replicante_models_core::actions::orchestrator::OrchestratorAction;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicante_models_core::events::Event;
use replicore_iface_orchestrator_action::OrchestratorActionRegistryEntry;

use super::utils::emit_event;
use super::utils::fail_action;
use crate::errors::SyncError;
use crate::ClusterOrchestrate;
use crate::ClusterOrchestrateMut;

/// Logic to start a pending/progress a running orchestration action.
pub fn run_action(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    action_record: OrchestratorAction,
    action: &OrchestratorActionRegistryEntry,
) -> Result<OrchestratorActionState> {
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());

    // Progress the action and handle errors.
    let updates = match action.handler.progress(&action_record) {
        Err(error) => {
            fail_action(data, data_mut, action_record, error)?;
            return Ok(OrchestratorActionState::Failed);
        }
        Ok(updates) => updates,
    };

    // Check for action timeouts to prevent endless actions.
    if let Some(scheduled_ts) = action_record.scheduled_ts {
        let next_state = updates
            .as_ref()
            .map(|updates| &updates.state)
            .unwrap_or(&action_record.state);
        let now = chrono::Utc::now();
        let timeout = action_record.timeout.unwrap_or(action.metadata.timeout);
        let timeout =
            chrono::Duration::from_std(timeout).expect("timeout to convert to chrono::Duration");

        if next_state.is_running() && (now > scheduled_ts + timeout) {
            let error = anyhow::anyhow!(crate::errors::ActionError::timed_out(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                action_record.action_id,
            ));
            fail_action(data, data_mut, action_record, error)?;
            return Ok(OrchestratorActionState::Failed);
        }
    }

    // End early if progressing does not need to make changes to the record.
    let updates = match updates {
        None => return Ok(action_record.state),
        Some(updates) => updates,
    };

    // Grab a copy of the old record in case we need it to emit events.
    let action_record_old = if !updates.state.is_final() {
        Some(action_record.clone())
    } else {
        None
    };

    // Update the action record.
    let mut action_record = action_record;
    action_record.state = updates.state;
    action_record.state_payload = updates.state_payload;

    // Ensure running actions have a scheduled timestamp
    // (and use this opportunity to also correct missed records from the past).
    if action_record.scheduled_ts.is_none() && action_record.state.is_running() {
        action_record.scheduled_ts = chrono::Utc::now().into();
    }

    // If the new state is final ensure the action record is finished.
    if action_record.state.is_final() {
        action_record.finish(updates.state)
    }

    // Determine events to emit.
    if action_record.state.is_final() {
        let event = Event::builder()
            .action()
            .orchestrator_action_finished(action_record.clone());
        emit_event(data, data_mut, event)?;
    }
    if let Some(old) = action_record_old {
        let new = action_record.clone();
        let event = Event::builder()
            .action()
            .orchestrator_action_changed(old, new);
        emit_event(data, data_mut, event)?;
    }

    // Persist updated action record.
    let state_after = action_record.state;
    data.store
        .persist()
        .orchestrator_action(action_record, span_context)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::store_persist(&data.namespace.ns_id, &data.cluster_view.cluster_id)
        })?;
    Ok(state_after)
}

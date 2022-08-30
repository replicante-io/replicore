use anyhow::Context;
use anyhow::Result;
use uuid::Uuid;

use replicante_models_core::actions::orchestrator::OrchestratorAction;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicante_models_core::events::Event;
use replicante_stream_events::EmitMessage;

use crate::errors::SyncError;
use crate::ClusterOrchestrate;
use crate::ClusterOrchestrateMut;

/// Generic logic to wrap event emitting.
pub fn emit_event(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    event: Event,
) -> Result<()> {
    let code = event.code();
    let stream_key = event.entity_id().partition_key();
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    let event = EmitMessage::with(stream_key, event)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::event_emit(&data.namespace.ns_id, &data.cluster_view.cluster_id, code)
        })?
        .trace(span_context);
    data.events
        .emit(event)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::event_emit(&data.namespace.ns_id, &data.cluster_view.cluster_id, code)
        })?;
    Ok(())
}

/// Update an action to the failed state.
pub fn fail_action(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    mut action: OrchestratorAction,
    error: anyhow::Error,
) -> Result<()> {
    // Update the action record to failed with error information.
    action.finish(OrchestratorActionState::Failed);
    let error = replicore_util_errors::ErrorInfo::from(error);
    let error = serde_json::to_value(error).expect("unable to serialise error information");
    action.state_payload_error = Some(error);

    // Emit action failed event.
    let event = Event::builder()
        .action()
        .orchestrator_action_finished(action.clone());
    emit_event(data, data_mut, event)?;

    // Store the updated record.
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    data.store
        .persist()
        .orchestrator_action(action, span_context)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::store_persist(&data.namespace.ns_id, &data.cluster_view.cluster_id)
        })
        .map_err(anyhow::Error::from)
}

/// Load the full action record from the DB.
pub fn get_orchestrator_action(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    action_id: Uuid,
) -> Result<OrchestratorAction> {
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    data.store
        .orchestrator_action(&data.cluster_view.cluster_id, action_id)
        .get(span_context)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::store_read(&data.namespace.ns_id, &data.cluster_view.cluster_id)
        })?
        .ok_or_else(|| {
            SyncError::expected_orchestrator_action_not_found(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                action_id,
            )
        })
        .map_err(anyhow::Error::from)
}

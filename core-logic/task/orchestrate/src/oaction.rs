//! Orchestrate execution of orchestrator actions.
use anyhow::Result;

use replisdk::core::models::oaction::OAction;
use replisdk::core::models::oaction::OActionState;

use replicore_cluster_view::ClusterViewBuilder;
use replicore_context::Context;
use replicore_events::Event;
use replicore_oaction::OActionChangeValue;
use replicore_oaction::OActionChanges;

use crate::init::InitData;

/// Progress all already running orchestrator actions.
pub async fn progress(
    context: &Context,
    data: &InitData,
    cluster_new: &mut ClusterViewBuilder,
) -> Result<()> {
    for action in &data.cluster_current.oactions_unfinished {
        let action = (**action).clone();

        // Invoke the action logic to make progress.
        let action = if action.state.is_running() {
            execute(context, data, action).await?
        } else {
            action
        };

        // Carry over view of unfinished (updated) actions.
        if !action.state.is_final() {
            cluster_new.oaction(action)?;
        }
    }
    Ok(())
}

/// Execute an orchestrator action by invoking its handler and updating the record as needed.
///
/// Any events needed as a result of updates is also emitted by this method.
///
/// ## Panic
///
/// This method panics if the orchestrator action is not in a state that can be executed:
///
/// - The action is `PendingApprove`.
/// - The action is finished.
pub async fn execute(context: &Context, data: &InitData, action: OAction) -> Result<OAction> {
    // Sanity check the action state before it is processed.
    if matches!(action.state, OActionState::PendingApprove) {
        panic!("cannot execute orchestration action pending approval");
    }
    if action.state.is_final() {
        panic!("cannot execute finished orchestration action");
    }

    // Invoke the action and update based on results.
    let mut action = action;
    let changes = invoke(context, data, &action).await;
    match changes {
        Err(error) => {
            let error = replisdk::utils::error::into_json(error);
            let changes = OActionChanges::to(OActionState::Failed).error(error);
            update(context, data, &mut action, changes).await?;
        }
        Ok(changes)
            // Actions must move to a running or final state.
            if matches!(
                changes.state,
                OActionState::PendingApprove | OActionState::PendingSchedule,
            ) =>
        {
            let error = anyhow::anyhow!("orchestrator action moved to invalid state");
            let error = replisdk::utils::error::into_json(error);
            let changes = OActionChanges::to(OActionState::Failed).error(error);
            update(context, data, &mut action, changes).await?;
        }
        Ok(changes) => update(context, data, &mut action, changes).await?,
    };
    Ok(action)
}

/// Lookup the orchestrator action implementation and invoke it.
///
/// The main purpose of this method is to consolidate invocation errors into one call
/// for easier handling of state transition and status update.
async fn invoke(context: &Context, data: &InitData, action: &OAction) -> Result<OActionChanges> {
    let metadata = data.injector.oactions.lookup(&action.kind)?;
    metadata.handler.invoke(context, &action).await
}

/// Update the [`OAction`] record with the result from invoking the handler.
async fn update(
    context: &Context,
    data: &InitData,
    action: &mut OAction,
    changes: OActionChanges,
) -> Result<()> {
    // If the record does not change skip updates.
    let state_change = action.state != changes.state;
    let error_change = match &changes.error {
        OActionChangeValue::Remove if action.state_payload_error.is_none() => false,
        OActionChangeValue::Unchanged => false,
        OActionChangeValue::Update(error) => match &action.state_payload_error {
            Some(current) if error == current => false,
            _ => true,
        }
        _ => true,
    };
    let payload_change = match &changes.payload {
        OActionChangeValue::Remove if action.state_payload.is_none() => false,
        OActionChangeValue::Unchanged => false,
        OActionChangeValue::Update(payload) => match &action.state_payload {
            Some(current) if payload == current => false,
            _ => true,
        }
        _ => true,
    };
    if !(state_change || error_change || payload_change) {
        return Ok(());
    }

    // Update the action record based on the changes.
    action.phase_to(changes.state);
    match changes.error {
        OActionChangeValue::Remove => action.state_payload_error = None,
        OActionChangeValue::Update(payload) => action.state_payload_error = Some(payload),
        OActionChangeValue::Unchanged => (),
    };
    match changes.payload {
        OActionChangeValue::Remove => action.state_payload = None,
        OActionChangeValue::Update(payload) => action.state_payload = Some(payload),
        OActionChangeValue::Unchanged => (),
    };
    let action = action;

    // Emit an update event.
    let event = match action.state {
        OActionState::Cancelled => crate::constants::OACTION_CANCEL,
        OActionState::Done => crate::constants::OACTION_SUCCESS,
        OActionState::Failed => crate::constants::OACTION_FAIL,
        OActionState::Running => crate::constants::OACTION_UPDATE,
        _ => panic!("unexpected oaction state for update"),
    };
    let event = Event::new_with_payload(event, &action)?;
    data.injector.events.change(context, event).await?;

    // Persist updated action.
    data.injector.store.persist(context, action.clone()).await?;
    Ok(())
}

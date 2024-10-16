//! Orchestrate execution of orchestrator actions.
use anyhow::Result;

use replisdk::core::models::oaction::OAction;
use replisdk::core::models::oaction::OActionState;

use replicore_cluster_models::OrchestrateMode;
use replicore_context::Context;
use replicore_events::Event;
use replicore_oaction::OActionChangeValue;
use replicore_oaction::OActionChanges;
use replicore_oaction::OActionInvokeArgs;

use crate::sync::SyncData;

/// Progress all already running orchestrator actions.
///
/// This method returns a list of still unfinished actions after all running actions executed.
pub async fn progress(
    context: &Context,
    data: &SyncData,
    oactions_unfinished: Vec<OAction>,
) -> Result<Vec<OAction>> {
    let mut still_unfinished = Vec::new();
    for action in oactions_unfinished {
        // Invoke the action logic to make progress.
        let action = if action.state.is_running() {
            execute(context, data, action).await?
        } else {
            action
        };

        // Carry over view of unfinished (updated) actions.
        if !action.state.is_final() {
            still_unfinished.push(action.clone());
            data.cluster_new_mut().oaction(action)?;
        }
    }
    Ok(still_unfinished)
}

/// Schedule (start) pending actions once scheduling constraints are met.
///
/// To ensure more complex logic can be built on top of actions scheduling has some constraints
/// that ensure a more predictable outcome. For this, actions are viewed as either:
///
/// - Root actions: actions created by the user of by system components aside from other actions.
/// - Leaf actions: actions created by other actions.
///
/// The constraints then are:
///
/// - Pending root actions are scheduled only if there are no running actions.
/// - Pending leaf actions are scheduled only if there are no running actions aside for its lineage.
/// - Pending node-exclusive actions are scheduled only if there are no running node actions
///   (on top of the above constraints).
///
/// NOTE: root and leaf actions are likely future features.
///
/// NOTE: node-exclusive actions are likely future features.
pub async fn schedule(
    context: &Context,
    data: &SyncData,
    oactions_unfinished: Vec<OAction>,
) -> Result<Vec<OAction>> {
    // Skip scheduling if the cluster mode is not sync.
    if matches!(data.mode, OrchestrateMode::Observe) {
        slog::debug!(
            context.logger, "Skip orchestrator action scheduling when sync is in observe mode";
            "ns_id" => data.ns_id(),
            "cluster_id" => data.cluster_id(),
        );
        return Ok(oactions_unfinished);
    }

    // Skip scheduling checks if no action needs scheduling.
    let any_pending = oactions_unfinished
        .iter()
        .any(|action| matches!(action.state, OActionState::PendingSchedule));
    if !any_pending {
        slog::debug!(
            context.logger, "Skip orchestrator action scheduling with no pending actions";
            "ns_id" => &data.cluster_current.spec.ns_id,
            "cluster_id" => &data.cluster_current.spec.cluster_id,
        );
        return Ok(oactions_unfinished);
    }

    // Check unfinished actions for the next pending one.
    // -> Root actions are blocked by any other running action.
    let any_running = oactions_unfinished
        .iter()
        .any(|action| action.state.is_running());

    // Ensure unfinished actions are still returned at the end.
    let mut still_unfinished = Vec::new();
    let mut oactions_unfinished = oactions_unfinished.into_iter();

    for action in oactions_unfinished.by_ref() {
        if !matches!(action.state, OActionState::PendingSchedule) {
            still_unfinished.push(action);
            continue;
        }

        // Skip scheduling if the action violates any constants.
        if any_running {
            slog::debug!(
                context.logger,
                "Skip orchestrator action scheduling due to other running action(s)";
                "ns_id" => &data.cluster_current.spec.ns_id,
                "cluster_id" => &data.cluster_current.spec.cluster_id,
            );
            break;
        }

        // Execute the action and stop if it does not complete at once.
        let action = execute(context, data, action).await?;
        if !action.state.is_final() {
            still_unfinished.push(action);
            break;
        }
    }

    // Return all still unfinished actions once all possible scheduling is done.
    still_unfinished.extend(oactions_unfinished);
    Ok(still_unfinished)
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
async fn execute(context: &Context, data: &SyncData, action: OAction) -> Result<OAction> {
    // Sanity check the action state before it is processed.
    if matches!(action.state, OActionState::PendingApprove) {
        panic!("cannot execute orchestration action pending approval");
    }
    if action.state.is_final() {
        panic!("cannot execute finished orchestration action");
    }

    // Invoke the action and update based on results.
    let mut action = action;
    action.scheduled_ts = Some(
        action
            .scheduled_ts
            .unwrap_or_else(time::OffsetDateTime::now_utc),
    );
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
async fn invoke(context: &Context, data: &SyncData, action: &OAction) -> Result<OActionChanges> {
    let metadata = data.injector.oactions.lookup(&action.kind)?;
    let args = OActionInvokeArgs {
        action,
        discovery: &data.cluster_current.discovery,
        spec: &data.cluster_current.spec,
    };
    let mut changes = metadata.handler.invoke(context, &args).await?;

    // If the action is running check to see if it timed out.
    if changes.state.is_running() {
        let now = time::OffsetDateTime::now_utc();
        let scheduled = action.scheduled_ts.unwrap_or(now);
        let timeout = scheduled + metadata.timeout;
        if now > timeout {
            slog::info!(
                context.logger, "Orchestrator action timed out";
                "ns_id" => &action.ns_id,
                "cluster_id" => &action.cluster_id,
                "action_id" => %action.action_id,
            );

            let error = anyhow::anyhow!("orchestrator action timed out");
            let error = replisdk::utils::error::into_json(error);
            changes.state = OActionState::Failed;
            changes = changes.error(error);
        }
    }

    Ok(changes)
}

/// Update the [`OAction`] record with the result from invoking the handler.
async fn update(
    context: &Context,
    data: &SyncData,
    action: &mut OAction,
    changes: OActionChanges,
) -> Result<()> {
    // If the record does not change skip updates.
    let state_change = action.state != changes.state;
    let error_change = match &changes.error {
        OActionChangeValue::Remove if action.state_payload_error.is_none() => false,
        OActionChangeValue::Unchanged => false,
        OActionChangeValue::Update(error) => {
            matches!(&action.state_payload_error, Some(current) if error == current)
        }
        _ => true,
    };
    let payload_change = match &changes.payload {
        OActionChangeValue::Remove if action.state_payload.is_none() => false,
        OActionChangeValue::Unchanged => false,
        OActionChangeValue::Update(payload) => {
            matches!(&action.state_payload, Some(current) if payload == current)
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

    // Emit an update event.
    let action: &OAction = action;
    let event = match action.state {
        OActionState::Cancelled => crate::constants::OACTION_CANCEL,
        OActionState::Done => crate::constants::OACTION_SUCCESS,
        OActionState::Failed => crate::constants::OACTION_FAIL,
        OActionState::Running => crate::constants::OACTION_UPDATE,
        _ => panic!("unexpected oaction state for update"),
    };
    let event = Event::new_with_payload(event, action)?;
    data.injector.events.change(context, event).await?;

    // Persist updated action.
    data.injector.store.persist(context, action.clone()).await?;
    Ok(())
}

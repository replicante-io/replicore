use std::collections::HashSet;

use anyhow::Context;
use anyhow::Result;
use slog::debug;
use slog::info;
use uuid::Uuid;

use replicante_agent_client::Client;
use replicante_models_agent::actions::api::ActionScheduleRequest;
use replicante_models_core::actions::node::Action;
use replicante_models_core::actions::node::ActionState;
use replicante_models_core::actions::node::ActionSyncSummary;
use replicante_models_core::events::Event;
use replicante_util_failure::SerializableFail;

use crate::errors::SyncError;
use crate::metrics::NODE_ACTIONS_SYNCED;
use crate::ClusterOrchestrate;
use crate::ClusterOrchestrateMut;

const MAX_SCHEDULE_ATTEMPTS: i32 = 10;

/// Sync node `Action` records from the node.
pub fn sync_node_actions(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    client: &dyn Client,
    node_id: &str,
) -> Result<()> {
    // Step 1: fetch agent info.
    let remote_ids = fetch_remote_actions_ids(data, data_mut, client, node_id)?;

    // Step 2: sync agent with core.
    let mut synced_actions = 0.0;
    for action_id in &remote_ids {
        sync_agent_action(data, data_mut, client, node_id, *action_id).map_err(|error| {
            NODE_ACTIONS_SYNCED.observe(synced_actions);
            error
        })?;
        synced_actions += 1.0;
    }
    NODE_ACTIONS_SYNCED.observe(synced_actions);

    // Step 3: sync core with agent.
    let actions = match data.cluster_view.unfinished_actions_on_node(node_id) {
        Some(actions) => actions,
        None => {
            debug!(
                data.logger,
                "No unfinished actions for node to sync";
                "namespace" => &data.cluster_view.namespace,
                "cluster_id" => &data.cluster_view.cluster_id,
                "node_id" => node_id,
            );
            return Ok(());
        }
    };
    for action_summary in actions {
        // Skip actions processed while looking at agent actions.
        if remote_ids.contains(&action_summary.action_id) {
            continue;
        }
        sync_core_action(data, data_mut, client, node_id, action_summary)?;
    }
    Ok(())
}

/// Fetch all action IDs currently available on the node.
pub(super) fn fetch_remote_actions_ids(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    client: &dyn Client,
    node_id: &str,
) -> Result<Vec<Uuid>> {
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    let mut duplicate_ids = HashSet::new();
    let mut remote_ids = Vec::new();
    debug!(
        data.logger,
        "Retrieving action IDs from agent";
        "namespace" => &data.namespace.ns_id,
        "cluster_id" => &data.cluster_view.cluster_id,
        "node_id" => node_id,
    );

    let queue = client
        .actions_queue(span_context.clone())
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::client_response(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
                "actions-queue",
            )
        })?;
    let finished = client
        .actions_finished(span_context)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::client_response(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
                "actions-finished",
            )
        })?;
    for list in &[queue, finished] {
        for action in list {
            if !duplicate_ids.contains(&action.id) {
                duplicate_ids.insert(action.id);
                remote_ids.push(action.id);
            }
        }
    }
    Ok(remote_ids)
}

/// Sync a single action from an agent to core.
pub(super) fn sync_agent_action(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    client: &dyn Client,
    node_id: &str,
    action_id: Uuid,
) -> Result<()> {
    // Fetch and process the action.
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    let info = client.action_info(&action_id, span_context.clone());
    let info = match info {
        Err(ref error) if error.not_found() => return Ok(()),
        _ => info.map_err(failure::Fail::compat).with_context(|| {
            SyncError::client_response(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
                "action-info",
            )
        }),
    }?;
    let action_agent = Action::new(
        data.cluster_view.cluster_id.clone(),
        node_id.to_string(),
        info.action,
    );
    let action_db = data
        .store
        .action(data.cluster_view.cluster_id.clone(), action_id)
        .get(span_context.clone())
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::store_read_for_node(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
            )
        })?;

    // Emit action-related events.
    let emit_action_finished;
    // --> New or change events.
    match action_db {
        None => {
            emit_action_finished = true;
            let event = Event::builder().action().new_action(action_agent.clone());
            super::emit_event(data, data_mut, node_id, event)?;
        }
        Some(action_db) => {
            // Skip updates for already finished and for unchanged actions.
            if action_db.finished_ts.is_some() || action_agent == action_db {
                debug!(
                    data.logger,
                    "Skipping update of finished action";
                    "namespace" => &data.cluster_view.namespace,
                    "cluster_id" => &data.cluster_view.cluster_id,
                    "node_id" => node_id,
                    "action_id" => action_agent.action_id.to_string(),
                );
                return Ok(());
            }

            emit_action_finished = action_agent.finished_ts.is_some();
            let event = Event::builder()
                .action()
                .changed(action_db, action_agent.clone());
            super::emit_event(data, data_mut, node_id, event)?;
        }
    };

    // --> Action finished event (if action was not already finished).
    if emit_action_finished && action_agent.finished_ts.is_some() {
        let event = Event::builder().action().finished(action_agent.clone());
        super::emit_event(data, data_mut, node_id, event)?;
    }

    // --> Emit an action history event.
    let event = Event::builder().action().history(
        action_agent.cluster_id.clone(),
        action_agent.node_id.clone(),
        action_agent.action_id,
        action_agent.finished_ts,
        info.history,
    );
    super::emit_event(data, data_mut, node_id, event)?;

    // --> Update the new cluster view.
    if action_agent.finished_ts.is_none() {
        data_mut
            .new_cluster_view
            .action(ActionSyncSummary::from(&action_agent))
            .with_context(|| {
                SyncError::cluster_view_update(
                    &data.namespace.ns_id,
                    &data.cluster_view.cluster_id,
                    node_id,
                )
            })?;
    }

    // --> Persist the new action information.
    data.store
        .persist()
        .action(action_agent, span_context)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::store_persist_for_node(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
            )
        })
        .map_err(anyhow::Error::from)
}

/// Sync a single action from an agent to core.
pub(super) fn sync_core_action(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    client: &dyn Client,
    node_id: &str,
    action_summary: &ActionSyncSummary,
) -> Result<()> {
    // Append pending actions to the new cluster view as well.
    let pending_action = matches!(
        action_summary.state,
        ActionState::PendingApprove | ActionState::PendingSchedule
    );
    if pending_action {
        data_mut
            .new_cluster_view
            .action(action_summary.clone())
            .with_context(|| {
                SyncError::cluster_view_update(
                    &data.namespace.ns_id,
                    &data.cluster_view.cluster_id,
                    node_id,
                )
            })?;
    }

    // Process core actions based on their state.
    match action_summary.state {
        // Schedule pending actions with the agent.
        ActionState::PendingSchedule => {
            if data.sched_choices.block_node {
                debug!(
                    data.logger,
                    "Scheduling choices blocked node actions";
                    "namespace" => &data.cluster_view.namespace,
                    "cluster_id" => &data.cluster_view.cluster_id,
                    "node_id" => node_id,
                    "action_id" => action_summary.action_id.to_string(),
                );
                return Ok(());
            }
            core_action_schedule(data, data_mut, client, node_id, action_summary)
        }
        // Running actions no longer known to the agent are lost.
        state if state.is_running() => core_action_lost(data, data_mut, node_id, action_summary),
        // Skip all other actions.
        _ => Ok(()),
    }
}

/// Update a lost action in the store so we can stop thinking of it.
fn core_action_lost(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    node_id: &str,
    action_summary: &ActionSyncSummary,
) -> Result<()> {
    // Get the action to emit the event correctly.
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    let action = data
        .store
        .action(
            data.cluster_view.cluster_id.clone(),
            action_summary.action_id,
        )
        .get(span_context.clone())
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::store_read_for_node(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
            )
        })?;
    let mut action = match action {
        // Can't lose an action we don't know about.
        None => return Ok(()),
        Some(action) => action,
    };
    action.finish(ActionState::Lost);
    data_mut.report.node_action_lost();

    // Emit action lost event.
    let event = Event::builder().action().lost(action.clone());
    super::emit_event(data, data_mut, node_id, event)?;

    // Persist lost action.
    data.store
        .persist()
        .action(action, span_context)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::store_persist_for_node(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
            )
        })?;
    debug!(
        data.logger,
        "Found lost action";
        "namespace" => &data.cluster_view.namespace,
        "cluster_id" => &data.cluster_view.cluster_id,
        "node_id" => node_id,
        "action_id" => action_summary.action_id.to_string(),
    );
    Ok(())
}

/// Update a lost action in the store so we can stop thinking of it.
fn core_action_schedule(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    client: &dyn Client,
    node_id: &str,
    action_summary: &ActionSyncSummary,
) -> Result<()> {
    // Get the action to schedule.
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    let action = data
        .store
        .action(
            data.cluster_view.cluster_id.clone(),
            action_summary.action_id,
        )
        .get(span_context.clone())
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::store_read_for_node(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
            )
        })?
        .ok_or_else(|| {
            SyncError::expected_action_not_found(
                &data.cluster_view.namespace,
                &data.cluster_view.cluster_id,
                node_id,
                action_summary.action_id,
            )
        })?;

    // Schedule the action with the agent.
    let request = ActionScheduleRequest {
        action_id: Some(action.action_id),
        args: action.args.clone(),
        created_ts: Some(action.created_ts),
        requester: Some(action.requester.clone()),
    };
    let result =
        client.schedule_action(&action.kind, &action.headers, request, span_context.clone());
    data_mut.report.node_action_scheduled();
    crate::metrics::NODE_ACTION_SCHEDULE_TOTAL.inc();

    // Handle scheduling error to re-try later.
    match result {
        Err(error) if error.kind().is_duplicate_action() => {
            info!(
                data.logger,
                "Ignored duplicate action scheduling attempt";
                "namespace" => &data.cluster_view.namespace,
                "cluster_id" => &data.cluster_view.cluster_id,
                "node_id" => node_id,
                "action_id" => action.action_id.to_string(),
            );
            crate::metrics::NODE_ACTION_SCHEDULE_DUPLICATE.inc();
        }
        Err(error) => {
            let old_action = action.clone();
            let mut action = action;

            // Record the failure for user debugging.
            let payload = SerializableFail::from(&error);
            let payload = serde_json::to_value(payload).expect("errors must always serialise");
            action.schedule_attempt += 1;
            action.state_payload = Some(payload);

            // After a while give up on trying to schedule actions.
            // TODO: make MAX_SCHEDULE_ATTEMPTS a namespace configuration once namespaces exist.
            if action.schedule_attempt > MAX_SCHEDULE_ATTEMPTS {
                action.finish(ActionState::Failed);
            }

            // Emit an action changed event so the error is tracked.
            let event = Event::builder()
                .action()
                .changed(old_action, action.clone());
            super::emit_event(data, data_mut, node_id, event)?;

            // Persist the error info to the DB.
            data.store
                .persist()
                .action(action, span_context)
                .map_err(failure::Fail::compat)
                .with_context(|| {
                    SyncError::store_persist_for_node(
                        &data.namespace.ns_id,
                        &data.cluster_view.cluster_id,
                        node_id,
                    )
                })?;
            crate::metrics::NODE_ACTION_SCHEDULE_ERROR.inc();
            data_mut.report.node_action_schedule_failed();
            let error = failure::Fail::compat(error);
            let error = anyhow::Error::from(error).context(SyncError::client_response(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
                "action-info",
            ));
            anyhow::bail!(error);
        }
        _ => (),
    };

    // On scheduling success reset attempt counter, if needed.
    if action.schedule_attempt != 0 {
        let mut action = action;
        action.schedule_attempt = 0;
        action.state_payload = None;
        data.store
            .persist()
            .action(action, span_context)
            .map_err(failure::Fail::compat)
            .with_context(|| {
                SyncError::store_persist_for_node(
                    &data.namespace.ns_id,
                    &data.cluster_view.cluster_id,
                    node_id,
                )
            })?;
    }
    Ok(())
}

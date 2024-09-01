//! Orchestrate scheduling of node actions.
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as Json;

use replisdk::agent::models::ActionExecutionRequest;
use replisdk::core::models::naction::NActionPhase;
use replisdk::platform::models::ClusterDiscoveryNode;

use replicore_cluster_models::OrchestrateMode;
use replicore_context::Context;

use crate::sync::SyncData;

/// Track the state of scheduling errors to report to user and prevent infinite attempts.
#[derive(Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct SchedulingErrorState {
    /// Number of scheduling attempts made so far.
    attempts: u16,

    /// Details of the last known scheduling error.
    last_error: Json,
}

/// Schedule ready to start node actions to nodes.
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
pub async fn schedule(context: &Context, data: &SyncData) -> Result<()> {
    // Skip scheduling if the cluster mode is not sync.
    if matches!(data.mode, OrchestrateMode::Observe) {
        slog::debug!(
            context.logger, "Skip node actions scheduling when sync is in observe mode";
            "ns_id" => data.ns_id(),
            "cluster_id" => &data.cluster_id(),
        );
        return Ok(());
    }

    // Skip scheduling if nodes are blocked by running orchestration actions.
    let any_oaction_running = data
        .cluster_current
        .oactions_unfinished
        .iter()
        .any(|action| action.state.is_running());
    if any_oaction_running {
        slog::debug!(
            context.logger, "Skip node actions scheduling due to running orchestrator actions";
            "ns_id" => data.ns_id(),
            "cluster_id" => &data.cluster_id(),
        );
        return Ok(());
    }

    // Process each node and schedule as needed.
    for node in &data.cluster_current.discovery.nodes {
        schedule_node(context, data, node).await?;
    }
    Ok(())
}

async fn schedule_node(
    context: &Context,
    data: &SyncData,
    node: &ClusterDiscoveryNode,
) -> Result<()> {
    // NOTE: on the use of cluster_new.
    //  While the new cluster view is incomplete at this stage, it does hold the just updated
    //  state of all nodes and node actions and is therefore the place to look.
    //  This for example ensures actions that were scheduled between last sync and now
    //  are not re-tried incorrectly.

    // Skip scheduling if no action needs scheduling.
    let unfinished = data
        .cluster_new_mut()
        .unfinished_node_actions(&node.node_id);
    let action = match unfinished.first() {
        Some(action) if matches!(action.state.phase, NActionPhase::PendingSchedule) => action,
        Some(_) | None => {
            slog::debug!(
                context.logger, "Skip node actions scheduling with no pending actions";
                "ns_id" => data.ns_id(),
                "cluster_id" => &data.cluster_id(),
                "node_id" => &node.node_id,
            );
            return Ok(());
        }
    };

    // Submit the action to the node and update its status.
    let client = data
        .injector
        .clients
        .agent
        .factory(context, &data.ns, &data.cluster_current.spec, node)
        .await?;
    let request = ActionExecutionRequest {
        args: action.args.clone(),
        created_time: Some(action.created_time),
        id: Some(action.action_id),
        kind: action.kind.clone(),
        metadata: action.metadata.clone(),
    };
    let error = match client.action_schedule(request).await {
        Ok(_) => return Ok(()),
        Err(error) => error,
    };

    // If the action is a duplicate assume it was sent successfully, next sync will update it.
    if error.is::<repliagent_client::ScheduleActionDuplicateId>() {
        return Ok(());
    }

    // Check and update scheduling error state.
    let mut state = match action.state.error.clone() {
        None => SchedulingErrorState::default(),
        Some(state) => serde_json::from_value(state).unwrap_or_default(),
    };
    state.attempts += 1;
    state.last_error = replisdk::utils::error::into_json(error);
    let phase = match state.attempts > data.ns.settings.orchestrate.max_naction_schedule_attempts {
        true => NActionPhase::Failed,
        false => action.state.phase,
    };

    let mut action = action.as_ref().clone();
    action.state.error = Some(serde_json::to_value(state)?);
    action.phase_to(phase);

    // Update cluster view and store with the updated record.
    match action.state.phase.is_final() {
        true => data.cluster_new_mut().remove_naction(&action)?,
        false => data.cluster_new_mut().update_naction(action.clone())?,
    };
    data.injector.store.persist(context, action).await?;
    Ok(())
}

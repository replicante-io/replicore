//! Synchronise node action information from nodes with the control plane.
use std::collections::HashSet;

use anyhow::Result;
use uuid::Uuid;

use replisdk::core::models::naction::NAction;
use replisdk::core::models::naction::NActionPhase;
use replisdk::platform::models::ClusterDiscoveryNode;

use repliagent_client::Client;
use replicore_cluster_models::OrchestrateReportNote;
use replicore_context::Context;
use replicore_events::Event;

use super::error::NodeSpecificError;
use crate::sync::SyncData;

/// Sync finished and queued node actions for a node.
pub async fn sync(
    context: &Context,
    data: &SyncData,
    node: &ClusterDiscoveryNode,
    client: &Client,
) -> Result<()> {
    // Get the list of unfinished (running or pending) node actions from the cluster view.
    let unfinished_ids: HashSet<Uuid> = data
        .cluster_current
        .unfinished_node_actions(&node.node_id)
        .iter()
        .map(|action| action.action_id)
        .collect();

    // List finished and queued action IDs from the node.
    let finished_ids: HashSet<Uuid> = client
        .actions_finished()
        .await
        .map_err(NodeSpecificError::from)?
        .actions
        .into_iter()
        .map(|entry| entry.id)
        .collect();
    let queue_id: HashSet<Uuid> = client
        .actions_queue()
        .await
        .map_err(NodeSpecificError::from)?
        .actions
        .into_iter()
        .map(|entry| entry.id)
        .collect();

    // Process actions from the node.
    let action_ids = finished_ids.intersection(&unfinished_ids);
    for action_id in action_ids {
        let action_id = *action_id;
        let node_id = node.node_id.clone();
        sync_action(context, data, node_id, client, action_id).await?;
    }
    for action_id in &queue_id {
        let action_id = *action_id;
        let node_id = node.node_id.clone();
        sync_action(context, data, node_id, client, action_id).await?;
    }

    // Handle lost actions (running in core but not reported by the agent).
    let all_node_ids: HashSet<Uuid> = finished_ids.union(&queue_id).copied().collect();
    let maybe_lost = unfinished_ids.difference(&all_node_ids);
    for action_id in maybe_lost {
        let action = match data.cluster_current.lookup_node_action(action_id) {
            Some(action) => action,
            None => continue,
        };
        let phase = match action.state.phase {
            // We think running but the agent didn't return => lost.
            phase if phase.is_running() => NActionPhase::Lost,
            // No finished action should be in the cluster view but just in case, ignore them.
            phase if phase.is_final() => continue,
            // Finally, pending actions are carried over to the new cluster view.
            _ => {
                data.cluster_new_mut().naction(action.clone())?;
                continue;
            }
        };
        let mut action = action.clone();
        action.phase_to(phase);
        persist(context, data, action).await?;
    }

    Ok(())
}

// Fetch full action details from the node.
async fn fetch(
    data: &SyncData,
    client: &Client,
    node_id: &str,
    action_id: Uuid,
) -> Result<NAction> {
    let action = client
        .action_lookup(action_id)
        .await
        .map_err(NodeSpecificError::from)?;
    let action = NAction {
        ns_id: data.ns_id().to_string(),
        cluster_id: data.cluster_id().to_string(),
        node_id: node_id.to_string(),
        action_id,
        args: action.args,
        created_time: action.created_time,
        finished_time: action.finished_time,
        kind: action.kind,
        metadata: action.metadata,
        scheduled_time: Some(action.scheduled_time),
        state: action.state.into(),
    };
    Ok(action)
}

/// Persist a [`NAction`] record.
///
/// - Adds the node action to the cluster view builder if unfinished.
/// - Emits associated events.
/// - Persist node action record to the store.
async fn persist(context: &Context, data: &SyncData, action: NAction) -> Result<()> {
    let action_id = &action.action_id;
    let current = data.cluster_current.index_nactions_by_id.get(action_id);

    // Emit node sync event as appropriate.
    let code = match current {
        Some(current) if current.as_ref() != &action => Some(crate::constants::NACTION_SYNC_UPDATE),
        None => Some(crate::constants::NACTION_SYNC_NEW),
        _ => None,
    };
    if let Some(code) = code {
        let event = Event::new_with_payload(code, action.clone())?;
        data.injector.events.change(context, event).await?;
    }

    // Update view and store.
    if !action.state.phase.is_final() {
        data.cluster_new_mut().naction(action.clone())?;
    }
    data.injector.store.persist(context, action).await?;
    Ok(())
}

async fn sync_action(
    context: &Context,
    data: &SyncData,
    node_id: String,
    client: &Client,
    action_id: Uuid,
) -> Result<()> {
    let action = match fetch(data, client, &node_id, action_id).await {
        Ok(action) => action,
        Err(error) => {
            let message = "Skipped node action sync due to error fetching details from the node";
            let mut note = OrchestrateReportNote::error(message, error);
            note.for_node(node_id).for_node_action(action_id);
            data.report_mut().notes.push(note);
            return Ok(());
        }
    };
    persist(context, data, action).await
}

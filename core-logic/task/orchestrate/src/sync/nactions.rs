//! Synchronise node action information from nodes with the control plane.
use std::collections::HashSet;

use anyhow::Context as AnyContext;
use anyhow::Result;
use uuid::Uuid;

use replisdk::core::models::naction::NAction;
use replisdk::core::models::naction::NActionPhase;
use replisdk::platform::models::ClusterDiscoveryNode;

use repliagent_client::Client;
use replicore_cluster_view::ClusterViewBuilder;
use replicore_context::Context;
use replicore_events::Event;

use super::error::NodeSpecificError;
use crate::init::InitData;

/// Sync finished and queued node actions for a node.
pub async fn sync(
    context: &Context,
    data: &InitData,
    cluster_new: &mut ClusterViewBuilder,
    node: &ClusterDiscoveryNode,
    client: &Client,
) -> Result<()> {
    // Get the list of pending node actions from the cluster view.
    let pending_action_ids = data.cluster_current.unfinished_node_actions();

    // List finished and queued action IDs from the node.
    let finished_ids: HashSet<Uuid> = client
        .actions_finished()
        .await
        .context(NodeSpecificError)?
        .actions
        .into_iter()
        .map(|entry| entry.id)
        .collect();
    let queue_id: HashSet<Uuid> = client
        .actions_queue()
        .await
        .context(NodeSpecificError)?
        .actions
        .into_iter()
        .map(|entry| entry.id)
        .collect();

    // Process actions from the node.
    let action_ids = finished_ids.intersection(&pending_action_ids);
    for action_id in action_ids {
        let action_id = *action_id;
        let node_id = node.node_id.clone();
        sync_action(context, data, cluster_new, node_id, client, action_id).await?;
    }

    for action_id in &queue_id {
        let action_id = *action_id;
        let node_id = node.node_id.clone();
        sync_action(context, data, cluster_new, node_id, client, action_id).await?;
    }

    // Handle lost actions (unfinished in core but not reported by the agent).
    let all_node_ids: HashSet<Uuid> = finished_ids.union(&queue_id).copied().collect();
    let lost = pending_action_ids.difference(&all_node_ids);
    for action_id in lost {
        let mut action = match data.cluster_current.lookup_node_action(action_id) {
            Some(action) => action.clone(),
            None => continue,
        };
        let phase = match action.state.phase {
            phase if phase.is_final() => continue,
            phase if phase.is_running() => NActionPhase::Lost,
            _ => NActionPhase::Cancelled,
        };
        action.phase_to(phase);
        persist(context, data, cluster_new, action).await?;
    }

    Ok(())
}

// Fetch full action details from the node.
async fn fetch(
    cluster_new: &mut ClusterViewBuilder,
    client: &Client,
    node_id: String,
    action_id: Uuid,
) -> Result<NAction> {
    let action = client
        .action_lookup(action_id)
        .await
        .context(NodeSpecificError)?;
    let action = NAction {
        ns_id: cluster_new.ns_id().to_string(),
        cluster_id: cluster_new.cluster_id().to_string(),
        node_id,
        action_id,
        args: action.args,
        created_time: action.created_time,
        finished_time: action.finished_time,
        kind: action.kind,
        metadata: action.metadata,
        scheduled_time: action.scheduled_time,
        state: action.state.into(),
    };
    Ok(action)
}

/// Persist a [`NAction`] record.
///
/// - Adds the node action to the cluster view builder if unfinished.
/// - Emits associated events.
/// - Persist node action record to the store.
async fn persist(
    context: &Context,
    data: &InitData,
    cluster_new: &mut ClusterViewBuilder,
    action: NAction,
) -> Result<()> {
    let action_id = &action.action_id;
    let node_id = &action.node_id;
    let current = data
        .cluster_current
        .nactions_by_node
        .get(node_id)
        .and_then(|actions| actions.get(action_id));

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
        cluster_new.node_action(action.clone())?;
    }
    data.injector.store.persist(context, action).await?;
    Ok(())
}

async fn sync_action(
    context: &Context,
    data: &InitData,
    cluster_new: &mut ClusterViewBuilder,
    node_id: String,
    client: &Client,
    action_id: Uuid,
) -> Result<()> {
    let action = match fetch(cluster_new, client, node_id, action_id).await {
        Ok(action) => action,
        Err(_error) => {
            // TODO: add to report as event.
            return Ok(());
        }
    };
    // TODO: handle failed action sync ... how?
    persist(context, data, cluster_new, action).await
}

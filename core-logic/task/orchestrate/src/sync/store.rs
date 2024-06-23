//! Persist store information from agents.
use anyhow::Result;

use replisdk::core::models::node::StoreExtras;
use replisdk::platform::models::ClusterDiscoveryNode;

use replicore_cluster_view::ClusterViewBuilder;
use replicore_context::Context;
use replicore_events::Event;

use crate::init::InitData;

/// Logic for persisting StoreExtra information about cluster nodes.
///
/// - Adds the info to the cluster view builder.
/// - Emits associated events.
/// - Persist record to the store.
pub async fn persist_extras(
    context: &Context,
    data: &InitData,
    cluster_new: &mut ClusterViewBuilder,
    node: &ClusterDiscoveryNode,
    extras: StoreExtras,
) -> Result<()> {
    let node_id = &node.node_id;

    // Emit sync event as appropriate.
    let code = match data.cluster_current.store_extras.get(node_id) {
        Some(current) if current.as_ref() != &extras => {
            Some(crate::constants::STORE_EXTRAS_SYNC_UPDATE)
        }
        None => Some(crate::constants::STORE_EXTRAS_SYNC_NEW),
        _ => None,
    };
    if let Some(code) = code {
        let event = Event::new_with_payload(code, extras.clone())?;
        data.injector.events.change(context, event).await?;
    }

    // Update view and store.
    cluster_new.store_extras(extras.clone())?;
    data.injector.store.persist(context, extras).await?;
    Ok(())
}

/// Update existing records to mark as stale.
pub async fn stale_extras(
    context: &Context,
    data: &InitData,
    cluster_new: &mut ClusterViewBuilder,
    node: &ClusterDiscoveryNode,
) -> Result<()> {
    // Only mark as stale if we have a StoreExtras record for the node.
    let extras = match data.cluster_current.store_extras.get(&node.node_id) {
        None => return Ok(()),
        Some(extras) => extras,
    };

    // If the extras are already stale there is nothing to do.
    let mut extras = extras.as_ref().clone();
    if !extras.fresh {
        cluster_new.store_extras(extras)?;
        return Ok(());
    }

    // Mark extras as stale and store them back.
    extras.fresh = false;
    persist_extras(context, data, cluster_new, node, extras).await
}

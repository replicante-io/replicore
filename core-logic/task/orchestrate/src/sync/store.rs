//! Persist store information from agents.
use anyhow::Result;

use replisdk::core::models::node::Shard;
use replisdk::core::models::node::StoreExtras;
use replisdk::platform::models::ClusterDiscoveryNode;

use replicore_context::Context;
use replicore_events::Event;

use crate::sync::SyncData;

/// Logic for persisting StoreExtra information about cluster nodes.
///
/// - Adds the info to the cluster view builder.
/// - Emits associated events.
/// - Persist record to the store.
pub async fn persist_extras(context: &Context, data: &SyncData, extras: StoreExtras) -> Result<()> {
    let node_id = &extras.node_id;

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
    data.cluster_new_mut().store_extras(extras.clone())?;
    data.injector.store.persist(context, extras).await?;
    Ok(())
}

/// Logic for persisting Shard information about cluster nodes.
///
/// - Adds the info to the cluster view builder.
/// - Emits associated events.
/// - Persist record to the store.
pub async fn persist_shards(context: &Context, data: &SyncData, shards: Vec<Shard>) -> Result<()> {
    for shard in shards {
        persist_shard(context, data, shard).await?;
    }
    Ok(())
}

/// Update existing StoreExtras records to mark as stale.
pub async fn stale_extras(
    context: &Context,
    data: &SyncData,
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
        data.cluster_new_mut().store_extras(extras)?;
        return Ok(());
    }

    // Mark extras as stale and store them back.
    extras.fresh = false;
    persist_extras(context, data, extras).await
}

/// Update existing [`Shard`] records to mark as stale.
pub async fn stale_shards(
    context: &Context,
    data: &SyncData,
    node: &ClusterDiscoveryNode,
) -> Result<()> {
    // Only mark as stale if we have Shard records for the node.
    let shards = match data.cluster_current.shards.get(&node.node_id) {
        None => return Ok(()),
        Some(extras) => extras,
    };

    for shard in shards.values() {
        // If the shard is already stale there is nothing to do.
        let mut shard = shard.as_ref().clone();
        if !shard.fresh {
            data.cluster_new_mut().shard(shard)?;
            continue;
        }

        // Mark shard as stale and store them back.
        shard.fresh = false;
        persist_shard(context, data, shard).await?;
    }

    Ok(())
}

/// Persist an individual shard as described by [`persist_shards`].
async fn persist_shard(context: &Context, data: &SyncData, shard: Shard) -> Result<()> {
    let node_id = &shard.node_id;
    let shard_id = &shard.shard_id;
    let current = data
        .cluster_current
        .shards
        .get(node_id)
        .and_then(|shards| shards.get(shard_id));

    // Emit sync event as appropriate.
    let code = match current {
        Some(current) if !current.same(&shard) => Some(crate::constants::SHARD_SYNC_UPDATE),
        None => Some(crate::constants::SHARD_SYNC_NEW),
        _ => None,
    };
    if let Some(code) = code {
        let event = Event::new_with_payload(code, shard.clone())?;
        data.injector.events.change(context, event).await?;
    }

    // Update view and store.
    data.cluster_new_mut().shard(shard.clone())?;
    data.injector.store.persist(context, shard).await?;
    Ok(())
}

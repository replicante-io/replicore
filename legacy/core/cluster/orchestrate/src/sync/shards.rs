use anyhow::Context;
use anyhow::Result;

use replicante_agent_client::Client;
use replicante_models_core::agent::Shard;
use replicante_models_core::events::Event;

use super::emit_event;
use crate::errors::SyncError;
use crate::ClusterOrchestrate;
use crate::ClusterOrchestrateMut;

/// Sync `Shard` records from the node.
pub fn sync_shards(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    client: &dyn Client,
    node_id: &str,
) -> Result<()> {
    // Grab shards from the node.
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    let info = client
        .shards(span_context)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::client_response(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
                "shards",
            )
        })?;

    // Sync each shard returned by the node.
    for shard in info.shards {
        let shard = Shard::new(
            data.cluster_view.cluster_id.clone(),
            node_id.to_string(),
            shard,
        );
        data_mut
            .new_cluster_view
            .shard(shard.clone())
            .with_context(|| {
                SyncError::cluster_view_update(
                    &data.namespace.ns_id,
                    &data.cluster_view.cluster_id,
                    node_id,
                )
            })?;
        let old = data
            .cluster_view
            .shard_on_node(&shard.node_id, &shard.shard_id)
            .cloned();
        match old {
            None => shard_new(data, data_mut, node_id, shard)?,
            Some(old) => shard_update(data, data_mut, node_id, old, shard)?,
        };
    }
    Ok(())
}

/// Checks if the "stable" attributes of a shard have changed.
///
/// Because shard data includes commit offsets and lag we need to do a more
/// in-depth comparison to ignore expected changes.
fn shard_attributes_changed(old: &Shard, new: &Shard) -> bool {
    new.cluster_id != old.cluster_id
        || new.node_id != old.node_id
        || new.role != old.role
        || new.shard_id != old.shard_id
}

/// Emit the event and persist the new/updated `Shard`.
fn shard_emit_and_persist<E>(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    node_id: &str,
    event: E,
    new: Shard,
) -> Result<()>
where
    E: Into<Option<Event>>,
{
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    if let Some(event) = event.into() {
        emit_event(data, data_mut, node_id, event)?;
    }
    data.store
        .persist()
        .shard(new, span_context)
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

/// Persist a new `Shard` record.
fn shard_new(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    node_id: &str,
    new: Shard,
) -> Result<()> {
    let event = Event::builder().shard().new_allocation(new.clone());
    shard_emit_and_persist(data, data_mut, node_id, event, new)
}

/// Persist a update to an existing `Shard` record.
fn shard_update(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    node_id: &str,
    old: Shard,
    new: Shard,
) -> Result<()> {
    if new == old {
        return Ok(());
    }
    // Skip events if the only thing that changed are offset info.
    let event = if shard_attributes_changed(&old, &new) {
        Some(
            Event::builder()
                .shard()
                .allocation_changed(old, new.clone()),
        )
    } else {
        None
    };
    shard_emit_and_persist(data, data_mut, node_id, event, new)
}

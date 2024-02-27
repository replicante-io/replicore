use anyhow::Context;
use anyhow::Result;
use slog::debug;

use replicante_models_core::agent::CommitOffset;
use replicante_models_core::agent::CommitUnit;
use replicante_models_core::agent::Shard;
use replicante_models_core::agent::ShardRole;
use replicante_models_core::events::Event;

use crate::errors::SyncError;
use crate::ClusterAggregateExtra;
use crate::ClusterAggregateExtraMut;
use crate::ClusterOrchestrate;

/// Synthesise secondary shard lag information when agents don't report it themselves.
pub fn synthesise(
    data: &ClusterOrchestrate,
    extra: &ClusterAggregateExtra,
    data_mut: &mut ClusterAggregateExtraMut,
) -> Result<()> {
    // Filter out primary shards.
    let secondaries = extra
        .new_cluster_view
        .shards
        .iter()
        .filter(|shard| shard.role != ShardRole::Primary)
        .filter(|shard| shard.lag.is_none());
    for shard in secondaries {
        synthesise_for_shard(data, extra, data_mut, shard)?;
    }
    Ok(())
}

/// Synthesise lag information for the given shard.
pub fn synthesise_for_shard(
    data: &ClusterOrchestrate,
    extra: &ClusterAggregateExtra,
    data_mut: &mut ClusterAggregateExtraMut,
    shard: &Shard,
) -> Result<()> {
    let cluster = &extra.new_cluster_view;

    // Find the primary for the shard.
    let primary = match cluster.shard_primary(&shard.shard_id) {
        Ok(Some(shard)) => shard,
        Ok(None) => {
            debug!(
                data.logger, "Skipping lag aggregation for shard without primary";
                "namespace" => &cluster.namespace,
                "cluster_id" => &cluster.cluster_id,
                "shard_id" => &shard.shard_id,
            );
            return Ok(());
        }
        Err(error) if error.is::<replicore_cluster_view::ManyPrimariesFound>() => {
            debug!(
                data.logger, "Skipping lag aggregation for shard with more then one primary";
                "namespace" => &cluster.namespace,
                "cluster_id" => &cluster.cluster_id,
                "shard_id" => &shard.shard_id,
            );
            return Ok(());
        }
        Err(error) => return Err(error),
    };

    // Ensure needed offsets are available, units match and units are supported.
    let (primary_offset, shard_offset) = match (&primary.commit_offset, &shard.commit_offset) {
        (Some(primary_offset), Some(shard_offset)) => (primary_offset, shard_offset),
        _ => {
            debug!(
                data.logger, "Skipping lag aggregation for shard without required offsets";
                "namespace" => &cluster.namespace,
                "cluster_id" => &cluster.cluster_id,
                "shard_id" => &shard.shard_id,
            );
            return Ok(());
        }
    };
    if primary_offset.unit != shard_offset.unit {
        debug!(
            data.logger, "Skipping lag aggregation for shard with different commit offset units";
            "namespace" => &cluster.namespace,
            "cluster_id" => &cluster.cluster_id,
            "shard_id" => &shard.shard_id,
        );
        return Ok(());
    }
    if !matches!(primary_offset.unit, CommitUnit::Seconds) {
        debug!(
            data.logger, "Skipping lag aggregation for shard with unsupported commit offset unit";
            "namespace" => &cluster.namespace,
            "cluster_id" => &cluster.cluster_id,
            "shard_id" => &shard.shard_id,
        );
        return Ok(());
    }

    // Compute shard lag for supported units.
    let lag = CommitOffset::new(
        primary_offset.value - shard_offset.value,
        primary_offset.unit.clone(),
    );
    let shard_updated = {
        let mut shard_updated = shard.clone();
        shard_updated.lag = Some(lag);
        shard_updated
    };

    // Events are emitted even if lag is the only change so the view DB can see the synthesised lag.
    // This is in contrast with changes to commit offset and lag handling during cluster sync.
    let event = Event::builder()
        .shard()
        .allocation_changed(shard.clone(), shard_updated.clone());
    super::emit_event(data, data_mut, event)?;

    // Persist updated shard to the store.
    super::check_lock(data, data_mut)?;
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    data.store
        .persist()
        .shard(shard_updated, span_context)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::store_persist(&data.namespace.ns_id, &data.cluster_view.cluster_id)
        })
        .map_err(anyhow::Error::from)
}

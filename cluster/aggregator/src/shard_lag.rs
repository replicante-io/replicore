use failure::Fail;
use failure::ResultExt;
use opentracingrust::Span;
use slog::debug;
use slog::Logger;

use replicante_models_core::agent::CommitOffset;
use replicante_models_core::agent::CommitUnit;
use replicante_models_core::agent::Shard;
use replicante_models_core::agent::ShardRole;
use replicante_models_core::events::Event;
use replicante_store_primary::store::Store;
use replicante_stream_events::EmitMessage;
use replicante_stream_events::Stream as EventsStream;
use replicore_cluster_view::ClusterView;
use replicore_util_errors::AnyWrap;

use crate::Error;
use crate::ErrorKind;
use crate::Result;

/// Synthesise secondary shard lag information when agents don't report it themselves.
pub fn synthesise(
    cluster: ClusterView,
    logger: &Logger,
    store: &Store,
    events: &EventsStream,
    span: &mut Span,
) -> Result<()> {
    // Filter out primary shards.
    let secondaries = cluster
        .shards
        .iter()
        .filter(|shard| shard.role != ShardRole::Primary)
        .filter(|shard| shard.lag.is_none());

    for shard in secondaries {
        synthesise_for_shard(shard, &cluster, logger, store, events, span)?;
    }

    Ok(())
}

/// Synthesise lag information for the given shard.
pub fn synthesise_for_shard(
    shard: &Shard,
    cluster: &ClusterView,
    logger: &Logger,
    store: &Store,
    events: &EventsStream,
    span: &mut Span,
) -> Result<()> {
    // Find the primary for the shard.
    let primary = match cluster.shard_primary(&shard.shard_id) {
        Ok(Some(shard)) => shard,
        Ok(None) => {
            debug!(
                logger, "Skipping lag aggregation for shard without primary";
                "namespace" => &cluster.namespace,
                "cluster_id" => &cluster.cluster_id,
                "shard_id" => &shard.shard_id,
            );
            return Ok(());
        }
        Err(error) if error.is::<replicore_cluster_view::ManyPrimariesFound>() => {
            debug!(
                logger, "Skipping lag aggregation for shard with too many primaries";
                "namespace" => &cluster.namespace,
                "cluster_id" => &cluster.cluster_id,
                "shard_id" => &shard.shard_id,
            );
            return Ok(());
        }
        Err(error) => {
            let error = AnyWrap::from(error)
                .context(ErrorKind::InvalidClusterState)
                .into();
            return Err(error);
        }
    };

    // Ensure needed offsets are available, units match and units are supported.
    let (primary_offset, shard_offset) = match (&primary.commit_offset, &shard.commit_offset) {
        (Some(primary_offset), Some(shard_offset)) => (primary_offset, shard_offset),
        _ => {
            debug!(
                logger, "Skipping lag aggregation for shard without required offsets";
                "namespace" => &cluster.namespace,
                "cluster_id" => &cluster.cluster_id,
                "shard_id" => &shard.shard_id,
            );
            return Ok(());
        }
    };
    if primary_offset.unit != shard_offset.unit {
        debug!(
            logger, "Skipping lag aggregation for shard with different commit offset units";
            "namespace" => &cluster.namespace,
            "cluster_id" => &cluster.cluster_id,
            "shard_id" => &shard.shard_id,
        );
        return Ok(());
    }
    if !matches!(primary_offset.unit, CommitUnit::Seconds) {
        debug!(
            logger, "Skipping lag aggregation for shard with unsupported commit offset unit";
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
    let code = event.code();
    let stream_key = event.entity_id().partition_key();
    let event = EmitMessage::with(stream_key, event)
        .with_context(|_| ErrorKind::EventEmit(code))?
        .trace(span.context().clone());
    events
        .emit(event)
        .with_context(|_| ErrorKind::EventEmit(code))?;

    // Persist updated shard to the primary store.
    store
        .persist()
        .shard(shard_updated, span.context().clone())
        .with_context(|_| ErrorKind::StoreWrite("shard update"))
        .map_err(Error::from)
}

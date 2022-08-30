use anyhow::Context;
use anyhow::Result;
use opentracingrust::Log;
use slog::debug;
use slog::warn;

use replicante_models_core::events::Event;
use replicante_stream_events::EmitMessage;

mod cluster_meta;
mod shard_lag;

use crate::errors::OperationError;
use crate::errors::SyncError;
use crate::ClusterAggregateExtra;
use crate::ClusterAggregateExtraMut;
use crate::ClusterOrchestrate;

/// Process aggregations for a cluster.
pub fn aggregate_cluster(
    data: &ClusterOrchestrate,
    extra: &ClusterAggregateExtra,
    data_mut: &mut ClusterAggregateExtraMut,
) -> Result<()> {
    debug!(
        data.logger, "Aggregating cluster";
        "cluster_id" => &data.cluster_view.cluster_id,
    );
    if let Some(span) = data_mut.span.as_mut() {
        span.log(Log::new().log("stage", "aggregate"));
    }
    check_lock(data, data_mut)?;

    // Aggregate cluster meta.
    self::cluster_meta::aggregate(data, extra, data_mut)?;

    // Synthesise lag metrics for secondary shard without them.
    self::shard_lag::synthesise(data, extra, data_mut)?;

    // Generated all aggregations.
    Ok(())
}

/// Exit early if lock was lost.
fn check_lock(data: &ClusterOrchestrate, data_mut: &mut ClusterAggregateExtraMut) -> Result<()> {
    if data.lock.inspect() {
        return Ok(());
    }

    if let Some(span) = data_mut.span.as_mut() {
        span.log(Log::new().log("abandoned", "lock lost"));
    }
    let ns_id = &data.namespace.ns_id;
    let cluster_id = &data.cluster_view.cluster_id;
    warn!(
        data.logger,
        "Cluster orchestrate lock lost";
        "namespace" => ns_id,
        "cluster_id" => cluster_id,
    );
    anyhow::bail!(OperationError::lock_lost(ns_id, cluster_id))
}

/// Generic logic to wrap event emitting.
fn emit_event(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterAggregateExtraMut,
    event: Event,
) -> Result<()> {
    let code = event.code();
    let stream_key = event.entity_id().partition_key();
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    let event = EmitMessage::with(stream_key, event)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::event_emit(&data.namespace.ns_id, &data.cluster_view.cluster_id, code)
        })?
        .trace(span_context);
    data.events
        .emit(event)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::event_emit(&data.namespace.ns_id, &data.cluster_view.cluster_id, code)
        })?;
    Ok(())
}

use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use opentracingrust::Log;
use slog::debug;
use slog::warn;

use replicante_agent_client::HttpClient;
use replicante_models_core::events::Event;
use replicante_stream_events::EmitMessage;

mod actions;
mod info;
mod shards;

#[cfg(test)]
mod tests;

use crate::errors::OperationError;
use crate::errors::OrchestratorEnder;
use crate::errors::SyncError;
use crate::ClusterOrchestrate;
use crate::ClusterOrchestrateMut;

/// Process every node in the cluster discovery record and sync the core state.
///
/// For each node in the cluster the following tasks are performed:
///
/// 1. Fetch updated information from the cluster nodes.
/// 2. Use this information to create an incremental updated view.
/// 3. Use this information and the starting cluster view to emit events.
/// 4. The information is also saved to the primary store.
/// 5. Schedule actions not already scheduled.
pub fn sync_cluster(data: &ClusterOrchestrate, data_mut: &mut ClusterOrchestrateMut) -> Result<()> {
    if let Some(span) = data_mut.span.as_mut() {
        span.log(Log::new().log("stage", "sync-cluster"));
    }

    // Sync each nodes in the cluster discovery.
    for node in &data.cluster_view.discovery.nodes {
        let result = sync_node(data, data_mut, node);
        let result = result.orchestration_failed()?;
        if let Err(error) = result {
            data_mut.report.node_failed();
            // TODO(dependency-updates): resume capturing in sentry as debug event.
            //let mut event = sentry::integrations::failure::event_from_fail(&error);
            //event.level = sentry::Level::Debug;
            //sentry::capture_event(event);
            debug!(
                data.logger,
                "Node-only error during sync for cluster orchestration";
                "namespace" => &data.cluster_view.namespace,
                "cluster_id" => &data.cluster_view.cluster_id,
                "agent_id" => &node,
                "error_detail" => ?error,
            );
            if let Some(span) = data_mut.span.as_mut() {
                // TODO(open-telemetry): Proper error tagging.
                span.log(Log::new().log("node.error", error.to_string()));
            }
        }
    }
    Ok(())
}

/// Sync an individual node in the cluster.
pub fn sync_node(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    node_id: &str,
) -> Result<()> {
    // Exit early if lock was lost.
    if !data.lock.inspect() {
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
            "node_id" => node_id,
        );
        anyhow::bail!(OperationError::lock_lost(ns_id, cluster_id))
    }

    // Count nodes that start syncing in orchestrate report.
    data_mut.report.node_syncing();

    // Client to interact with the node API.
    // TODO(node-client-refactor): replace specific HTTP client with a protocol agnostic option.
    // NOTE:
    //  How to decide which client protocol to Instantiate?
    //  Maybe this can become part of a dedicated crate to lookup the protocol and pick the impl?
    //  Similar patterns to consider: interfaces, orchestrator actions, ...
    let client = HttpClient::new(
        &data.namespace,
        node_id.to_string(),
        data.node_timeout,
        data.logger.clone(),
        Arc::clone(&data.tracer),
    )
    .map_err(failure::Fail::compat)
    .with_context(|| {
        SyncError::client_connect(
            &data.namespace.ns_id,
            &data.cluster_view.cluster_id,
            node_id,
        )
    })?;

    // Sync agent and node information.
    // The agent overall status depends on both the agent and node statues.
    // To ensure information is determined and stored correctly we first attempt to grab
    // all the info about the agent and then derive the overall status based on previous errors.
    let agent_info = self::info::sync_agent_info(data, data_mut, &client, node_id);
    let node_info = self::info::sync_node_info(data, data_mut, &client, node_id);
    self::info::sync_agent(data, data_mut, node_id, agent_info, node_info)?;

    // If the node is up sync shard and agent information.
    self::shards::sync_shards(data, data_mut, &client, node_id)?;
    self::actions::sync_node_actions(data, data_mut, &client, node_id)?;
    Ok(())
}

/// Generic logic to wrap event emitting.
fn emit_event(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    node_id: &str,
    event: Event,
) -> Result<()> {
    let code = event.code();
    let stream_key = event.entity_id().partition_key();
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    let event = EmitMessage::with(stream_key, event)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::event_emit_for_node(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
                code,
            )
        })?
        .trace(span_context);
    data.events
        .emit(event)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::event_emit_for_node(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                node_id,
                code,
            )
        })?;
    Ok(())
}

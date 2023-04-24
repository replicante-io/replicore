use std::collections::HashSet;

use anyhow::Context;
use anyhow::Result;

use replicante_models_core::agent::ShardRole;
use replicante_models_core::cluster::ClusterMeta;

use crate::errors::SyncError;
use crate::ClusterAggregateExtra;
use crate::ClusterAggregateExtraMut;
use crate::ClusterOrchestrate;

/// Aggregate cluster metadata from a `ClusterView`.
pub fn aggregate(
    data: &ClusterOrchestrate,
    extra: &ClusterAggregateExtra,
    data_mut: &mut ClusterAggregateExtraMut,
) -> Result<()> {
    let cluster_view = &extra.new_cluster_view;
    let cluster_id = cluster_view.cluster_id.clone();
    // TODO(remove-display-name): Remove display name logic once attribute moves to ClusterSettings.
    let cluster_display_name = cluster_id.clone();

    let mut meta = ClusterMeta::new(cluster_id, cluster_display_name);
    meta.nodes = cluster_view.nodes.len() as i32;
    meta.shards_count = cluster_view.unique_shards_count() as i32;

    // Count agents and nodes reporting as down.
    for agent in cluster_view.agents.values() {
        if !agent.status.is_up() {
            meta.agents_down += 1;
        }
        if agent.status.is_node_down() {
            meta.nodes_down += 1;
        }
    }

    // List all kinds of nodes.
    let kinds: HashSet<&String> = cluster_view.nodes.values().map(|node| &node.kind).collect();
    meta.kinds = kinds.into_iter().cloned().collect();

    // Check shard primaries.
    for shard in &cluster_view.shards {
        if matches!(shard.role, ShardRole::Primary) {
            meta.shards_primaries += 1;
        }
    }

    // Persist ClusterMeta to the primary store.
    super::check_lock(data, data_mut)?;
    let span_context = data_mut.span.as_ref().map(|span| span.context().clone());
    data.store
        .legacy()
        .persist_cluster_meta(meta, span_context)
        .map_err(failure::Fail::compat)
        .with_context(|| {
            SyncError::store_persist(&data.namespace.ns_id, &data.cluster_view.cluster_id)
        })
        .map_err(anyhow::Error::from)
}

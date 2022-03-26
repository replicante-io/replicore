use std::collections::HashSet;

use replicante_models_core::agent::ShardRole;
use replicante_models_core::cluster::ClusterMeta;
use replicore_cluster_view::ClusterView;

use super::Result;

/// Aggregate cluster metadata from a ClusterView.
pub(crate) fn aggregate(cluster_view: &ClusterView) -> Result<ClusterMeta> {
    let cluster_id = cluster_view.cluster_id.clone();
    let cluster_display_name = cluster_view
        .discovery
        .display_name
        .clone()
        .unwrap_or_else(|| cluster_id.clone());

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
    let kinds: HashSet<&String> = cluster_view
        .nodes
        .iter()
        .map(|(_, node)| &node.kind)
        .collect();
    meta.kinds = kinds.into_iter().cloned().collect();

    // Check shard primaries.
    for shard in &cluster_view.shards {
        if matches!(shard.role, ShardRole::Primary) {
            meta.shards_primaries += 1;
        }
    }

    Ok(meta)
}

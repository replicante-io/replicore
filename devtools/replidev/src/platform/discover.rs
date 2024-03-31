use std::collections::hash_map::Entry;
use std::collections::HashMap;

use anyhow::Result;
use replisdk::platform::models::ClusterDiscovery;
use replisdk::platform::models::ClusterDiscoveryNode;
use replisdk::platform::models::ClusterDiscoveryResponse;

use super::Platform;

const DEFAULT_NODE_CLASS: &str = "play.node";

/// Discover nodes running in podman and group them as clusters.
pub async fn discover(platform: &Platform) -> Result<ClusterDiscoveryResponse> {
    // List all running nodes.
    let nodes = super::node_list::list_nodes(&platform.conf)
        .await
        .map_err(replisdk::utils::actix::error::Error::from)?;

    // Format nodes into cluster discover records.
    let mut clusters: HashMap<String, ClusterDiscovery> = HashMap::new();
    for node_pod in nodes {
        let cluster_id = node_pod.cluster;
        let node_id = node_pod.node;
        let agent_address = match node_pod.port_agent {
            Some(port) => format!("https://{}:{}", platform.agents_address, port),
            None => continue,
        };
        let node = ClusterDiscoveryNode {
            agent_address,
            node_class: node_pod
                .node_class
                .unwrap_or_else(|| DEFAULT_NODE_CLASS.to_string()),
            node_id,
            node_group: node_pod.node_group,
        };
        match clusters.entry(cluster_id) {
            Entry::Occupied(mut entry) => entry.get_mut().nodes.push(node),
            Entry::Vacant(entry) => {
                let cluster = ClusterDiscovery {
                    cluster_id: entry.key().clone(),
                    nodes: vec![node],
                };
                entry.insert(cluster);
            }
        };
    }
    Ok(ClusterDiscoveryResponse {
        clusters: clusters.into_values().collect(),
    })
}

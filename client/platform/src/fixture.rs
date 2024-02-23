//! Mock client implementation for unit tests.
use std::collections::HashMap;
use std::sync::Mutex;

use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::Rng;

use replisdk::platform::models::ClusterDiscovery;
use replisdk::platform::models::ClusterDiscoveryNode;
use replisdk::platform::models::ClusterDiscoveryResponse;
use replisdk::platform::models::NodeDeprovisionRequest;
use replisdk::platform::models::NodeProvisionRequest;
use replisdk::platform::models::NodeProvisionResponse;

/// Mock client implementation for unit tests.
#[derive(Default)]
pub struct Client {
    state: Mutex<ClientState>,
}

impl Client {
    /// Append a node to a cluster so it can be discovered.
    ///
    /// Clusters are auto-created the first node is created.
    pub fn append_node<S1, S2>(&self, cluster: S1, node: S2)
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let cluster = cluster.into();
        let node = node.into();
        let node = ClusterDiscoveryNode {
            agent_address: format!("unittest://{}", node),
            node_id: node.clone(),
        };
        self.state
            .lock()
            .unwrap()
            .clusters
            .entry(cluster)
            .or_default()
            .push(node)
    }
}

#[async_trait::async_trait]
impl super::IPlatform for Client {
    async fn deprovision(&self, request: NodeDeprovisionRequest) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        let mut empty = false;
        state.clusters.get_mut(&request.cluster_id).map(|nodes| {
            let index = nodes
                .iter()
                .position(|node| node.node_id == request.node_id);
            if let Some(index) = index {
                nodes.remove(index);
            }
            if nodes.is_empty() {
                empty = true;
            }
        });
        if empty {
            state.clusters.remove(&request.cluster_id);
        }
        Ok(())
    }

    async fn discover(&self) -> Result<ClusterDiscoveryResponse> {
        let state = self.state.lock().unwrap();
        let clusters = state.clusters.clone();
        let clusters: Vec<ClusterDiscovery> = clusters
            .into_iter()
            .map(|(cluster_id, nodes)| ClusterDiscovery { cluster_id, nodes })
            .collect();
        Ok(ClusterDiscoveryResponse { clusters })
    }

    async fn provision(&self, request: NodeProvisionRequest) -> Result<NodeProvisionResponse> {
        let count = request
            .cluster
            .nodes
            .get(&request.provision.node_group_id)
            .unwrap()
            .desired_count;
        let node_ids: Vec<String> = (0..count)
            .map(|_| {
                rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(12)
                    .map(char::from)
                    .collect::<String>()
            })
            .collect();
        for node in &node_ids {
            self.append_node(&request.cluster.cluster_id, node);
        }
        let response = NodeProvisionResponse {
            count,
            node_ids: Some(node_ids),
        };
        Ok(response)
    }
}

/// Internal state to implement platform mocking.
#[derive(Default)]
struct ClientState {
    clusters: HashMap<String, Vec<ClusterDiscoveryNode>>,
}

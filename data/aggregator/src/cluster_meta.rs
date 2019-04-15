use std::collections::HashSet;

use failure::ResultExt;

use replicante_coordinator::NonBlockingLockWatcher;
use replicante_data_models::Agent;
use replicante_data_models::AgentInfo;
use replicante_data_models::AgentStatus;
use replicante_data_models::ClusterDiscovery;
use replicante_data_models::ClusterMeta;
use replicante_data_models::Node;
use replicante_data_models::Shard;
use replicante_data_store::Store;

use super::ErrorKind;
use super::Result;

pub(crate) struct ClusterMetaAggregator {
    lock: NonBlockingLockWatcher,
    store: Store,

    // Data used to build metadata model.
    agents_down: i32,
    cluster_display_name: Option<String>,
    cluster_id: Option<String>,
    kinds: HashSet<String>,
    nodes: i32,
    nodes_down: i32,
    shards: HashSet<String>,
}

impl ClusterMetaAggregator {
    /// Persist the generated metadata record.
    pub(crate) fn commit(mut self) -> Result<()> {
        // Validate required data.
        if self.cluster_id.is_none() {
            return Err(ErrorKind::MissingMetadata("cluster_id").into());
        }
        if self.cluster_display_name.is_none() {
            // A cluster_display_name is None if no Node was fetched from the cluster
            // (in case all nodes or agents are down).
            // In that case default to the cluster ID for the display name.
            self.cluster_display_name = self.cluster_id.clone();
        }

        // Build the model.
        let cluster_id = self.cluster_id.take().unwrap();
        let cluster_display_name = self.cluster_display_name.take().unwrap();
        let mut meta = ClusterMeta::new(cluster_id.clone(), cluster_display_name);
        meta.agents_down = self.agents_down;
        meta.kinds = self.kinds.into_iter().collect();
        meta.nodes = self.nodes;
        meta.nodes_down = self.nodes_down;
        meta.shards = self.shards.len() as i32;

        // Write back aggregation result.
        if !self.lock.inspect() {
            return Err(ErrorKind::ClusterLockLost(cluster_id).into());
        }
        self.store.persist_cluster_meta(meta)
            .with_context(|_| ErrorKind::StoreWrite("ClusterMeta"))?;
        Ok(())
    }

    pub(crate) fn new(store: Store, lock: NonBlockingLockWatcher) -> ClusterMetaAggregator {
        ClusterMetaAggregator {
            lock,
            store,

            // Data used to build metadata model.
            agents_down: 0,
            cluster_display_name: None,
            cluster_id: None,
            kinds: HashSet::new(),
            nodes: 0,
            nodes_down: 0,
            shards: HashSet::new(),
        }
    }

    /// Update the metadata record with info from the Agent model.
    pub(crate) fn visit_agent(&mut self, agent: &Agent) -> Result<()> {
        self.nodes += 1;
        match agent.status {
            AgentStatus::AgentDown(_) => self.agents_down += 1,
            AgentStatus::NodeDown(_) => self.nodes_down += 1,
            AgentStatus::Up => (),
        };
        Ok(())
    }

    /// Update the metadata record with info from the AgentInfo model.
    pub(crate) fn visit_agent_info(&mut self, _: &AgentInfo) -> Result<()> {
        Ok(())
    }

    /// Update the metadata record with info from the discovery model.
    pub(crate) fn visit_discovery(&mut self, discovery: &ClusterDiscovery) -> Result<()> {
        self.cluster_id = Some(discovery.cluster_id.to_string());
        Ok(())
    }

    /// Update the metadata record with info about a node.
    pub(crate) fn visit_node(&mut self, node: &Node) -> Result<()> {
        if self.cluster_display_name.is_none() {
            self.cluster_display_name = Some(node.cluster_display_name.clone());
        }
        self.kinds.insert(node.kind.clone());
        Ok(())
    }

    /// Update the metadata record with info about a shard.
    pub(crate) fn visit_shard(&mut self, shard: &Shard) -> Result<()> {
        self.shards.insert(shard.shard_id.clone());
        Ok(())
    }
}

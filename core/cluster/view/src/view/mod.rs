use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use anyhow::Result;
use serde::ser::SerializeStruct;

use replicante_models_core::actions::ActionSummary;
use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::Node;
use replicante_models_core::agent::Shard;
use replicante_models_core::agent::ShardRole;
use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::ClusterSettings;

use crate::ManyPrimariesFound;

mod builder;
mod refs;

#[cfg(test)]
mod tests;

pub use self::builder::ClusterViewBuilder;

/// Synthetic in-memory view of a Cluster.
#[derive(Debug)]
pub struct ClusterView {
    // Cluster identification attributes
    pub cluster_id: String,
    pub namespace: String,

    // Whole cluster records.
    pub discovery: ClusterDiscovery,
    pub settings: ClusterSettings,

    // Cluster entity records.
    pub actions_unfinished_by_node: HashMap<String, Vec<ActionSummary>>,
    pub agents: HashMap<String, Agent>,
    pub agents_info: HashMap<String, AgentInfo>,
    pub nodes: HashMap<String, Rc<Node>>,
    pub shards: Vec<Rc<Shard>>,

    // "Indexes" to access records in different ways.
    shards_by_id: HashMap<String, BTreeMap<String, Rc<Shard>>>,
    shards_by_node: HashMap<String, BTreeMap<String, Rc<Shard>>>,
}

impl ClusterView {
    /// Start building an empty `ClusterView` from required data.
    pub fn builder(
        settings: ClusterSettings,
        discovery: ClusterDiscovery,
    ) -> Result<ClusterViewBuilder> {
        ClusterViewBuilder::new(settings, discovery)
    }

    /// Lookup a specific node in the cluster.
    pub fn node(&self, node: &str) -> Option<&Node> {
        self.nodes.get(node).map(Deref::deref)
    }

    /// Lookup a specific shard on a node in the cluster.
    pub fn shard_on_node(&self, node: &str, shard: &str) -> Option<&Shard> {
        self.shards_by_node
            .get(node)
            .and_then(|node| node.get(shard).map(Deref::deref))
    }

    /// Lookup the primary shard record for a shard.
    ///
    /// If multiple shards with the given ID and primary role are found
    /// a `ManyPrimariesFound` error is return.
    pub fn shard_primary(&self, shard: &str) -> Result<Option<&Shard>> {
        let nodes = match self.shards_by_id.get(shard) {
            None => return Ok(None),
            Some(nodes) => nodes,
        };
        let primaries: Vec<&Shard> = nodes
            .values()
            .filter(|shard| shard.role == ShardRole::Primary)
            .map(Deref::deref)
            .collect();

        // Error if more then one primary is found.
        if primaries.len() > 1 {
            let error = ManyPrimariesFound {
                namespace: self.namespace.clone(),
                cluster_id: self.cluster_id.clone(),
                shard_id: shard.to_string(),
                records: primaries.into_iter().cloned().collect(),
            };
            anyhow::bail!(error);
        }

        // Or return the maybe one found.
        let mut primaries = primaries;
        let primary = primaries.pop();
        Ok(primary)
    }

    /// Lookup unfinished actions targeting a node.
    pub fn unfinished_actions_on_node(&self, node: &str) -> Option<&Vec<ActionSummary>> {
        self.actions_unfinished_by_node.get(node)
    }

    /// Count the number of unique shards by ID.
    pub fn unique_shards_count(&self) -> usize {
        self.shards_by_id.len()
    }
}

impl serde::Serialize for ClusterView {
    /// Serialise a ClusterView as a structured object.
    ///
    /// References to the same objects from "indexes" are serialised as the IDs of the
    /// referenced objects to avoid repeating the same objects multiple times.
    ///
    /// Additionally serialisation uses `BTreeMap`s to allow stable serialisation.
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("ClusterView", 11)?;
        state.serialize_field("cluster_id", &self.cluster_id)?;
        state.serialize_field("namespace", &self.namespace)?;
        state.serialize_field("settings", &self.settings)?;
        state.serialize_field("discovery", &self.discovery)?;

        // Convert HashMap to BTreeMap for stable serialisation.
        let actions_unfinished_by_node: BTreeMap<&String, &Vec<ActionSummary>> =
            self.actions_unfinished_by_node.iter().collect();
        let agents: BTreeMap<&String, &Agent> = self.agents.iter().collect();
        let agents_info: BTreeMap<&String, &AgentInfo> = self.agents_info.iter().collect();
        state.serialize_field("actions_unfinished_by_node", &actions_unfinished_by_node)?;
        state.serialize_field("agents", &agents)?;
        state.serialize_field("agents_info", &agents_info)?;

        // Translate maps to enable serialisation of `Rc` values.
        let nodes: BTreeMap<&String, &Node> =
            self.nodes.iter().map(|(k, v)| (k, v.as_ref())).collect();
        let shards: Vec<&Shard> = self.shards.iter().map(|s| s.as_ref()).collect();
        state.serialize_field("nodes", &nodes)?;
        state.serialize_field("shards", &shards)?;

        // Translate indexed maps to serialise IDs instead of full objects.
        let shards_by_id: BTreeMap<&String, BTreeMap<&String, refs::ShardRef>> = self
            .shards_by_id
            .iter()
            .map(|(shard_id, nodes)| {
                let mapped_nodes = nodes
                    .iter()
                    .map(|(node_id, info)| (node_id, info.as_ref().into()))
                    .collect();
                (shard_id, mapped_nodes)
            })
            .collect();
        let shards_by_node: BTreeMap<&String, BTreeMap<&String, refs::ShardRef>> = self
            .shards_by_node
            .iter()
            .map(|(node_id, shards)| {
                let mapped_shards = shards
                    .iter()
                    .map(|(shard_id, info)| (shard_id, info.as_ref().into()))
                    .collect();
                (node_id, mapped_shards)
            })
            .collect();

        state.serialize_field("shards_by_id", &shards_by_id)?;
        state.serialize_field("shards_by_node", &shards_by_node)?;
        state.end()
    }
}

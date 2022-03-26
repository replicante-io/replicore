use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;
use std::rc::Rc;

use anyhow::anyhow;
use anyhow::Result;
use serde::ser::SerializeStruct;

use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::Node;
use replicante_models_core::agent::Shard;
use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::ClusterSettings;

use crate::ClusterViewCorrupt;

mod refs;

#[cfg(test)]
mod tests;

/// Syntetic in-memory view of a Cluster.
#[derive(Debug)]
pub struct ClusterView {
    // Cluster identification attributes
    pub cluster_id: String,
    pub namespace: String,

    // Whole cluster records.
    pub discovery: ClusterDiscovery,
    pub settings: ClusterSettings,

    // Individual node records.
    pub agents: HashMap<String, Agent>,
    pub agents_info: HashMap<String, AgentInfo>,
    pub nodes: HashMap<String, Rc<Node>>,
    pub shards: Vec<Rc<Shard>>,

    // "Indexes" to access records in different ways.
    shards_by_id: HashMap<String, BTreeMap<String, Rc<Shard>>>,
    shards_by_node: HashMap<String, BTreeMap<String, Rc<Shard>>>,
}

impl ClusterView {
    /// Start building a Cluster view from essential data.
    pub fn builder(
        settings: ClusterSettings,
        discovery: ClusterDiscovery,
    ) -> Result<ClusterViewBuilder> {
        // Grab the namespace and cluster id from the settings.
        let namespace = settings.namespace.clone();
        let cluster_id = settings.cluster_id.clone();

        // Then make sure the discovery record is for the same cluster.
        // TODO(namespace-rollout): Validate namespaces match.
        if discovery.cluster_id != cluster_id {
            return Err(anyhow!(ClusterViewCorrupt::cluster_id_clash(
                namespace,
                cluster_id,
                discovery.cluster_id
            )));
        }

        let view = ClusterView {
            cluster_id,
            namespace,
            discovery,
            settings,
            agents: HashMap::new(),
            agents_info: HashMap::new(),
            nodes: HashMap::new(),
            shards: Vec::new(),
            shards_by_id: HashMap::new(),
            shards_by_node: HashMap::new(),
        };
        Ok(ClusterViewBuilder {
            seen_agents: HashSet::new(),
            seen_agents_info: HashSet::new(),
            seen_nodes: HashSet::new(),
            seen_shards: HashSet::new(),
            view,
        })
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

    /// Count the number of unique shards by ID.
    pub fn unique_shards_count(&self) -> usize {
        self.shards_by_id.len()
    }
}

impl serde::Serialize for ClusterView {
    /// Serialise a ClusterView as a structred object.
    ///
    /// References to the same objects from "indexes" are serialised as the IDs of the
    /// referenced objects to avoid repeating the same objects multiple times.
    ///
    /// Additionally serialisation uses `BTreeMap`s to allow stable serialisation.
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("ClusterView", 10)?;
        state.serialize_field("cluster_id", &self.cluster_id)?;
        state.serialize_field("namespace", &self.namespace)?;
        state.serialize_field("settings", &self.settings)?;
        state.serialize_field("discovery", &self.discovery)?;

        // Convert HashMap to BTreeMap for stable serialisation.
        let agents: BTreeMap<&String, &Agent> = self.agents.iter().collect();
        let agents_info: BTreeMap<&String, &AgentInfo> = self.agents_info.iter().collect();
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

/// Incrementally build a Cluster view from individual records.
pub struct ClusterViewBuilder {
    // Track cluster entiries already added to the view.
    seen_agents: HashSet<String>,
    seen_agents_info: HashSet<String>,
    seen_nodes: HashSet<String>,
    seen_shards: HashSet<(String, String)>,

    // Keep the incrementally built view ready to return.
    view: ClusterView,
}

impl ClusterViewBuilder {
    /// Add an Agent to the Cluster View.
    pub fn agent(&mut self, agent: Agent) -> Result<&mut Self> {
        // Can't add an agent from another cluster.
        if self.view.cluster_id != agent.cluster_id {
            return Err(anyhow!(ClusterViewCorrupt::cluster_id_clash(
                &self.view.namespace,
                &self.view.cluster_id,
                agent.cluster_id,
            )));
        }

        // Can't add the same agent twice.
        if self.seen_agents.contains(&agent.host) {
            return Err(anyhow!(ClusterViewCorrupt::duplicate_agent(
                &self.view.namespace,
                &self.view.cluster_id,
                agent.host,
            )));
        }

        self.seen_agents.insert(agent.host.clone());
        self.view.agents.insert(agent.host.clone(), agent);
        Ok(self)
    }

    /// Add Agent information to the Cluster View.
    pub fn agent_info(&mut self, info: AgentInfo) -> Result<&mut Self> {
        // Can't add agent info from another cluster.
        if self.view.cluster_id != info.cluster_id {
            return Err(anyhow!(ClusterViewCorrupt::cluster_id_clash(
                &self.view.namespace,
                &self.view.cluster_id,
                info.cluster_id,
            )));
        }

        // Can't add the same agent info twice.
        if self.seen_agents_info.contains(&info.host) {
            return Err(anyhow!(ClusterViewCorrupt::duplicate_agent_info(
                &self.view.namespace,
                &self.view.cluster_id,
                info.host,
            )));
        }

        self.seen_agents_info.insert(info.host.clone());
        self.view.agents_info.insert(info.host.clone(), info);
        Ok(self)
    }

    /// Convert this view builder into a complete ClusterView.
    pub fn build(self) -> ClusterView {
        self.view
    }

    /// Add node information to the Cluster View.
    pub fn node(&mut self, node: Node) -> Result<&mut Self> {
        // Can't add node from another cluster.
        if self.view.cluster_id != node.cluster_id {
            return Err(anyhow!(ClusterViewCorrupt::cluster_id_clash(
                &self.view.namespace,
                &self.view.cluster_id,
                node.cluster_id,
            )));
        }

        // Can't add the same node twice.
        if self.seen_nodes.contains(&node.node_id) {
            return Err(anyhow!(ClusterViewCorrupt::duplicate_node(
                &self.view.namespace,
                &self.view.cluster_id,
                node.node_id,
            )));
        }

        self.seen_nodes.insert(node.node_id.clone());
        self.view.nodes.insert(node.node_id.clone(), Rc::new(node));
        Ok(self)
    }

    /// Add shard information to the Cluster View.
    pub fn shard(&mut self, shard: Shard) -> Result<&mut Self> {
        // Can't add shard from another cluster.
        if self.view.cluster_id != shard.cluster_id {
            return Err(anyhow!(ClusterViewCorrupt::cluster_id_clash(
                &self.view.namespace,
                &self.view.cluster_id,
                shard.cluster_id,
            )));
        }

        // Can't add the same shard twice.
        let key = (shard.node_id.clone(), shard.shard_id.clone());
        if self.seen_shards.contains(&key) {
            return Err(anyhow!(ClusterViewCorrupt::duplicate_shard(
                &self.view.namespace,
                &self.view.cluster_id,
                shard.node_id,
                shard.shard_id,
            )));
        }

        let shard = Rc::new(shard);
        self.seen_shards.insert(key);
        self.view.shards.push(Rc::clone(&shard));

        self.view
            .shards_by_id
            .entry(shard.shard_id.clone())
            .or_insert_with(BTreeMap::new)
            .insert(shard.node_id.clone(), Rc::clone(&shard));
        self.view
            .shards_by_node
            .entry(shard.node_id.clone())
            .or_insert_with(BTreeMap::new)
            .insert(shard.shard_id.clone(), shard);
        Ok(self)
    }
}

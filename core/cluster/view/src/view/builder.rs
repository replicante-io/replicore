use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use anyhow::anyhow;
use anyhow::Result;
use uuid::Uuid;

use replicante_models_core::actions::node::ActionSyncSummary;
use replicante_models_core::actions::orchestrator::OrchestratorActionSyncSummary;
use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::Node;
use replicante_models_core::agent::Shard;
use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::ClusterSettings;

use crate::ClusterView;
use crate::ClusterViewCorrupt;

/// Incrementally build a Cluster view from individual records.
pub struct ClusterViewBuilder {
    // Track cluster entities already added to the view.
    seen_actions_by_node: HashMap<String, HashSet<Uuid>>,
    seen_actions_orchestrator: HashSet<Uuid>,
    seen_agents: HashSet<String>,
    seen_agents_info: HashSet<String>,
    seen_nodes: HashSet<String>,
    seen_shards: HashSet<(String, String)>,

    // Keep the incrementally built view ready to return.
    view: ClusterView,
}

impl ClusterViewBuilder {
    /// Start building an empty `ClusterView` from required data.
    pub(crate) fn new(
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
            actions_unfinished_by_node: HashMap::new(),
            actions_unfinished_orchestrator: Vec::new(),
            agents: HashMap::new(),
            agents_info: HashMap::new(),
            nodes: HashMap::new(),
            shards: Vec::new(),
            shards_by_id: HashMap::new(),
            shards_by_node: HashMap::new(),
        };
        Ok(ClusterViewBuilder {
            seen_actions_by_node: HashMap::new(),
            seen_actions_orchestrator: HashSet::new(),
            seen_agents: HashSet::new(),
            seen_agents_info: HashSet::new(),
            seen_nodes: HashSet::new(),
            seen_shards: HashSet::new(),
            view,
        })
    }

    /// Add an Action to the Cluster View.
    pub fn action(&mut self, summary: ActionSyncSummary) -> Result<&mut Self> {
        // Can't add an action from another cluster.
        if self.view.cluster_id != summary.cluster_id {
            return Err(anyhow!(ClusterViewCorrupt::cluster_id_clash(
                &self.view.namespace,
                &self.view.cluster_id,
                summary.cluster_id,
            )));
        }

        // Can't add the same action twice.
        let seen = match self.seen_actions_by_node.get(&summary.node_id) {
            None => false,
            Some(actions) => actions.contains(&summary.action_id),
        };
        if seen {
            return Err(anyhow!(ClusterViewCorrupt::duplicate_action(
                &self.view.namespace,
                &self.view.cluster_id,
                summary.node_id,
                summary.action_id,
            )));
        }
        self.seen_actions_by_node
            .entry(summary.node_id.clone())
            .or_insert_with(HashSet::new)
            .insert(summary.action_id);

        self.view
            .actions_unfinished_by_node
            .entry(summary.node_id.clone())
            .or_insert_with(Vec::new)
            .push(summary);
        Ok(self)
    }

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

    /// Add an OrchestratorAction to the Cluster View.
    pub fn orchestrator_action(
        &mut self,
        summary: OrchestratorActionSyncSummary,
    ) -> Result<&mut Self> {
        // Can't add an action for another cluster.
        if self.view.cluster_id != summary.cluster_id {
            return Err(anyhow!(ClusterViewCorrupt::cluster_id_clash(
                &self.view.namespace,
                &self.view.cluster_id,
                summary.cluster_id,
            )));
        }

        // Can't add the same orchestrator action twice.
        if self.seen_actions_orchestrator.contains(&summary.action_id) {
            return Err(anyhow!(ClusterViewCorrupt::duplicate_orchestrator_action(
                &self.view.namespace,
                &self.view.cluster_id,
                summary.action_id,
            )));
        }
        self.seen_actions_orchestrator.insert(summary.action_id);

        self.view.actions_unfinished_orchestrator.push(summary);
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

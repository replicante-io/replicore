use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use anyhow::anyhow;
use anyhow::Result;

use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::ClusterSettings;

use crate::ClusterViewCorrupt;

#[cfg(test)]
mod tests;

/// Syntetic in-memory view of a Cluster.
pub struct ClusterView {
    // Cluster identification attributes
    pub cluster_id: String,
    pub namespace: String,

    // Whole cluster records.
    pub discovery: ClusterDiscovery,
    pub settings: ClusterSettings,

    // Individual node records.
    pub agents: Vec<Rc<Agent>>,
    pub agents_info: HashMap<String, AgentInfo>,
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
            agents: Vec::new(),
            agents_info: HashMap::new(),
        };
        Ok(ClusterViewBuilder {
            seen_agents: HashSet::new(),
            seen_agents_info: HashSet::new(),
            view,
        })
    }
}

/// Incrementally build a Cluster view from individual records.
pub struct ClusterViewBuilder {
    // Track cluster entiries already added to the view.
    seen_agents: HashSet<String>,
    seen_agents_info: HashSet<String>,

    // Keep the incrementally built view ready to return.
    view: ClusterView,
}

impl ClusterViewBuilder {
    /// Add an Agent to the Cluster View.
    pub fn agent(&mut self, agent: Agent) -> Result<&mut Self> {
        // Can't add the same agent twice.
        if self.seen_agents.contains(&agent.host) {
            return Err(anyhow!(ClusterViewCorrupt::duplicate_agent(
                &self.view.namespace,
                &self.view.cluster_id,
                agent.host,
            )));
        }

        self.seen_agents.insert(agent.host.clone());
        self.view.agents.push(Rc::new(agent));
        Ok(self)
    }

    /// Add Agent information to the Cluster View.
    pub fn agent_info(&mut self, info: AgentInfo) -> Result<&mut Self> {
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
}

use replicante_data_models::Agent as AgentModel;
use replicante_data_models::AgentInfo as AgentInfoModel;
use replicante_data_models::ClusterDiscovery as ClusterDiscoveryModel;
use replicante_data_models::Node as NodeModel;
use replicante_data_models::Shard as ShardModel;

use super::super::backend::PersistImpl;
use super::super::Result;

/// Persist (insert or update) models to the store.
pub struct Persist {
    persist: PersistImpl,
}

impl Persist {
    pub(crate) fn new(persist: PersistImpl) -> Persist {
        Persist { persist }
    }

    /// Create or update an Agent record.
    pub fn agent(&self, agent: AgentModel) -> Result<()> {
        self.persist.agent(agent)
    }

    /// Create or update an AgentInfo record.
    pub fn agent_info(&self, agent: AgentInfoModel) -> Result<()> {
        self.persist.agent_info(agent)
    }

    /// Create or update a ClusterDiscovery record.
    pub fn cluster_discovery(&self, discovery: ClusterDiscoveryModel) -> Result<()> {
        self.persist.cluster_discovery(discovery)
    }

    /// Creat or update a Node record.
    pub fn node(&self, node: NodeModel) -> Result<()> {
        self.persist.node(node)
    }

    /// Creat or update a Shard record.
    pub fn shard(&self, shard: ShardModel) -> Result<()> {
        self.persist.shard(shard)
    }
}

use replicante_models_core::actions::Action;
use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::Node;
use replicante_models_core::agent::Shard;
use replicante_models_core::cluster::ClusterDiscovery;
use replicante_models_core::cluster::ClusterMeta;

use crate::backend::DataImpl;
use crate::Cursor;
use crate::Result;

/// Data validation operations.
pub struct Data {
    data: DataImpl,
}

impl Data {
    pub(crate) fn new(data: DataImpl) -> Data {
        Data { data }
    }

    /// Iterate over all actions in the store.
    pub fn actions(&self) -> Result<Cursor<Action>> {
        self.data.actions()
    }

    /// Iterate over all agents in the store.
    pub fn agents(&self) -> Result<Cursor<Agent>> {
        self.data.agents()
    }

    /// Iterate over all agents info in the store.
    pub fn agents_info(&self) -> Result<Cursor<AgentInfo>> {
        self.data.agents_info()
    }

    /// Iterate over all cluster discoveries in the store.
    pub fn cluster_discoveries(&self) -> Result<Cursor<ClusterDiscovery>> {
        self.data.cluster_discoveries()
    }

    /// Iterate over all cluster metadata in the store.
    pub fn clusters_meta(&self) -> Result<Cursor<ClusterMeta>> {
        self.data.clusters_meta()
    }

    /// Iterate over all nodes in the store.
    pub fn nodes(&self) -> Result<Cursor<Node>> {
        self.data.nodes()
    }

    /// Iterate over all shards in the store.
    pub fn shards(&self) -> Result<Cursor<Shard>> {
        self.data.shards()
    }
}

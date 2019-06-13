use replicante_models_core::Agent;
use replicante_models_core::AgentInfo;
use replicante_models_core::ClusterDiscovery;
use replicante_models_core::ClusterMeta;
use replicante_models_core::Event;
use replicante_models_core::Node;
use replicante_models_core::Shard;

use super::super::backend::DataImpl;
use super::super::Cursor;
use super::super::Result;

/// Data validation operations.
pub struct Data {
    data: DataImpl,
}

impl Data {
    pub(crate) fn new(data: DataImpl) -> Data {
        Data { data }
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

    /// Iterate over all events in the store.
    pub fn events(&self) -> Result<Cursor<Event>> {
        self.data.events()
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

use opentracingrust::SpanContext;

use replicante_models_core::actions::Action as ActionModel;
use replicante_models_core::agent::Agent as AgentModel;
use replicante_models_core::agent::AgentInfo as AgentInfoModel;
use replicante_models_core::agent::Node as NodeModel;
use replicante_models_core::agent::Shard as ShardModel;
use replicante_models_core::cluster::discovery::ClusterDiscovery as ClusterDiscoveryModel;
use replicante_models_core::cluster::discovery::DiscoverySettings;

use crate::backend::PersistImpl;
use crate::Result;

/// Persist (insert or update) models to the store.
pub struct Persist {
    persist: PersistImpl,
}

impl Persist {
    pub(crate) fn new(persist: PersistImpl) -> Persist {
        Persist { persist }
    }

    /// Create or update an agent `Action` record.
    pub fn action<S>(&self, action: ActionModel, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.action(action, span.into())
    }

    /// Create or update an Agent record.
    pub fn agent<S>(&self, agent: AgentModel, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.agent(agent, span.into())
    }

    /// Create or update an AgentInfo record.
    pub fn agent_info<S>(&self, agent: AgentInfoModel, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.agent_info(agent, span.into())
    }

    /// Create or update a ClusterDiscovery record.
    pub fn cluster_discovery<S>(&self, discovery: ClusterDiscoveryModel, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.cluster_discovery(discovery, span.into())
    }

    /// Create or update a cluster DiscoverySettings record.
    pub fn discovery_settings<S>(&self, settings: DiscoverySettings, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.discovery_settings(settings, span.into())
    }

    /// Creat or update a Node record.
    pub fn node<S>(&self, node: NodeModel, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.node(node, span.into())
    }

    /// Creat or update a Shard record.
    pub fn shard<S>(&self, shard: ShardModel, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.shard(shard, span.into())
    }
}

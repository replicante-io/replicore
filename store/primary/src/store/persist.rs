use opentracingrust::SpanContext;
use replisdk::platform::models::ClusterDiscovery as ClusterDiscoveryModel;

use replicante_models_core::actions::node::Action as ActionModel;
use replicante_models_core::actions::orchestrator::OrchestratorAction as OrchestratorActionModel;
use replicante_models_core::agent::Agent as AgentModel;
use replicante_models_core::agent::AgentInfo as AgentInfoModel;
use replicante_models_core::agent::Node as NodeModel;
use replicante_models_core::agent::Shard as ShardModel;
use replicante_models_core::cluster::discovery::DiscoverySettings;
use replicante_models_core::cluster::ClusterSettings;

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

    /// Create or update a ClusterSettings record.
    pub fn cluster_settings<S>(&self, settings: ClusterSettings, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.cluster_settings(settings, span.into())
    }

    /// Create or update a cluster DiscoverySettings record.
    pub fn discovery_settings<S>(&self, settings: DiscoverySettings, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.discovery_settings(settings, span.into())
    }

    /// Update the next_orchestrate of a ClusterSettings record.
    ///
    /// The new value is based on the current time + settings.interval.
    pub fn next_cluster_orchestrate<S>(&self, settings: ClusterSettings, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.next_cluster_orchestrate(settings, span.into())
    }

    /// Update the next_run of a DiscoverySettings record.
    ///
    /// The new value is based on the current time + settings.interval.
    pub fn next_discovery_run<S>(&self, settings: DiscoverySettings, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.next_discovery_run(settings, span.into())
    }

    /// Create or update a Node record.
    pub fn node<S>(&self, node: NodeModel, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.node(node, span.into())
    }

    /// Create or update an OrchestratorAction record.
    pub fn orchestrator_action<S>(&self, action: OrchestratorActionModel, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.orchestrator_action(action, span.into())
    }

    /// Create or update a Shard record.
    pub fn shard<S>(&self, shard: ShardModel, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.shard(shard, span.into())
    }
}

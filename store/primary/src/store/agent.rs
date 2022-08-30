use opentracingrust::SpanContext;

use replicante_models_core::agent::Agent as AgentModel;
use replicante_models_core::agent::AgentInfo as AgentInfoModel;

use crate::backend::AgentImpl;
use crate::Result;

/// Operate on the agent identified by the provided cluster_id and host.
pub struct Agent {
    agent: AgentImpl,
    attrs: AgentAttributes,
}

impl Agent {
    pub(crate) fn new(agent: AgentImpl, attrs: AgentAttributes) -> Agent {
        Agent { agent, attrs }
    }

    /// Query the `Agent` record, if any is stored.
    pub fn get<S>(&self, span: S) -> Result<Option<AgentModel>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.agent.get(&self.attrs, span.into())
    }

    /// Query the `AgentInfo` record, if any is stored.
    pub fn info<S>(&self, span: S) -> Result<Option<AgentInfoModel>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.agent.info(&self.attrs, span.into())
    }
}

/// Attributes attached to all `Agent` operations.
pub struct AgentAttributes {
    pub cluster_id: String,
    pub host: String,
}

use opentracingrust::SpanContext;

use replicante_models_core::agent::Agent as AgentModel;
use replicante_models_core::agent::AgentInfo as AgentInfoModel;

use crate::backend::AgentsImpl;
use crate::Cursor;
use crate::Result;

/// Operate on all agent in the cluster identified by cluster_id.
pub struct Agents {
    agents: AgentsImpl,
    attrs: AgentsAttribures,
}

impl Agents {
    pub(crate) fn new(agents: AgentsImpl, attrs: AgentsAttribures) -> Agents {
        Agents { agents, attrs }
    }

    /// Iterate over agents in a cluster.
    pub fn iter<S>(&self, span: S) -> Result<Cursor<AgentModel>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.agents.iter(&self.attrs, span.into())
    }

    /// Iterate over info for agents in a cluster.
    pub fn iter_info<S>(&self, span: S) -> Result<Cursor<AgentInfoModel>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.agents.iter_info(&self.attrs, span.into())
    }
}

/// Attributes attached to all `Agents` operations.
pub struct AgentsAttribures {
    pub cluster_id: String,
}

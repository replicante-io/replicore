use opentracingrust::SpanContext;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_data_models::Agent as AgentModel;
use replicante_data_models::AgentInfo as AgentInfoModel;

use super::super::backend::AgentsImpl;
use super::super::Cursor;
use super::super::Result;

/// Operate on all agent in the cluster identified by cluster_id.
pub struct Agents {
    agents: AgentsImpl,
    attrs: AgentsAttribures,
}

impl Agents {
    pub(crate) fn new(agents: AgentsImpl, attrs: AgentsAttribures) -> Agents {
        Agents { agents, attrs }
    }

    /// Count agents in the cluster.
    pub fn counts<S>(&self, span: S) -> Result<AgentsCounts>
    where
        S: Into<Option<SpanContext>>,
    {
        self.agents.counts(&self.attrs, span.into())
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

/// Counts returned by the `Agents::counts` operation.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct AgentsCounts {
    pub agents_down: i32,
    pub nodes: i32,
    pub nodes_down: i32,
}

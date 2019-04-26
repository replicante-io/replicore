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
    pub fn counts(&self) -> Result<AgentsCounts> {
        self.agents.counts(&self.attrs)
    }

    /// Iterate over agents in a cluster.
    pub fn iter(&self) -> Result<Cursor<AgentModel>> {
        self.agents.iter(&self.attrs)
    }

    /// Iterate over info for agents in a cluster.
    pub fn iter_info(&self) -> Result<Cursor<AgentInfoModel>> {
        self.agents.iter_info(&self.attrs)
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

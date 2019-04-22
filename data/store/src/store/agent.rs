use replicante_data_models::Agent as AgentModel;
use replicante_data_models::AgentInfo as AgentInfoModel;

use super::super::backend::AgentImpl;
use super::super::Result;

/// Operate on the agent identified by the provided cluster_id and host.
pub struct Agent {
    agent: AgentImpl,
    attrs: AgentAttribures,
}

impl Agent {
    pub(crate) fn new(agent: AgentImpl, attrs: AgentAttribures) -> Agent {
        Agent { agent, attrs }
    }

    /// Query the `Agent` record, if any is stored.
    pub fn get(&self) -> Result<Option<AgentModel>> {
        self.agent.get(&self.attrs)
    }

    /// Query the `AgentInfo` record, if any is stored.
    pub fn info(&self) -> Result<Option<AgentInfoModel>> {
        self.agent.info(&self.attrs)
    }
}

/// Attributes attached to all `Agent` operations.
pub struct AgentAttribures {
    pub cluster_id: String,
    pub host: String,
}

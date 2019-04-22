use replicante_data_models::AgentInfo as AgentInfoModel;

/// Status of an agent.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    #[serde(flatten)]
    pub agent: AgentInfoModel,

    /// If `true` the model was NOT updated by the last cluster state refresh operation.
    pub stale: bool,
}

impl From<AgentInfoModel> for AgentInfo {
    fn from(agent: AgentInfoModel) -> AgentInfo {
        AgentInfo {
            agent,
            stale: false,
        }
    }
}

impl From<AgentInfo> for AgentInfoModel {
    fn from(wrapper: AgentInfo) -> AgentInfoModel {
        wrapper.agent
    }
}

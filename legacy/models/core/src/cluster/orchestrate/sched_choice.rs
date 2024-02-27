use serde::Deserialize;
use serde::Serialize;

use crate::actions::orchestrator::OrchestratorActionScheduleMode;

/// Store computed orchestration scheduling choices.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub struct SchedChoice {
    /// Block scheduling of pending data store node actions.
    pub block_node: bool,

    /// Block scheduling (starting) of pending exclusive orchestration.
    pub block_orchestrator_exclusive: bool,

    /// Reasons combined to reach the current choice.
    pub reasons: Vec<SchedChoiceReason>,
}

impl SchedChoice {
    /// Check if a specific orchestrator action scheduling mode is blocked.
    pub fn is_mode_blocked(&self, mode: &OrchestratorActionScheduleMode) -> bool {
        match mode {
            OrchestratorActionScheduleMode::Exclusive => self.block_orchestrator_exclusive,
        }
    }
}

impl Default for SchedChoice {
    fn default() -> Self {
        SchedChoice {
            block_node: false,
            block_orchestrator_exclusive: false,
            reasons: vec![SchedChoiceReason::DefaultToAll],
        }
    }
}

/// List of scheduling choices made to reach the final choices.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum SchedChoiceReason {
    /// By default all actions can be scheduled.
    #[serde(rename = "DEFAULT_TO_ALL")]
    DefaultToAll,

    /// One or more pending schedule node actions were found.
    #[serde(rename = "FOUND_NODE_PENDING")]
    FoundNodePending,

    /// One or more running node actions were found.
    #[serde(rename = "FOUND_NODE_RUNNING")]
    FoundNodeRunning,

    /// One or more pending schedule exclusive orchestrator actions were found.
    #[serde(rename = "FOUND_ORCHESTRATOR_EXCLUSIVE_PENDING")]
    FoundOrchestratorExclusivePending,

    /// One or more running exclusive orchestrator actions were found.
    #[serde(rename = "FOUND_ORCHESTRATOR_EXCLUSIVE_RUNNING")]
    FoundOrchestratorExclusiveRunning,
}

#[cfg(test)]
mod tests {
    use super::SchedChoice;
    use crate::actions::orchestrator::OrchestratorActionScheduleMode;

    #[test]
    fn check_orchestrator_exclusive_blocked() {
        let choice = {
            let mut choice = SchedChoice::default();
            choice.block_orchestrator_exclusive = true;
            choice
        };
        let blocked = choice.is_mode_blocked(&OrchestratorActionScheduleMode::Exclusive);
        assert_eq!(blocked, true);
    }
}

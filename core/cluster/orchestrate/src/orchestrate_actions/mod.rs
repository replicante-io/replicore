use std::collections::HashSet;

use anyhow::Result;
use opentracingrust::Log;

use replicante_models_core::actions::orchestrator::OrchestratorActionState;

mod pending;
mod progress;
mod running;
mod utils;

#[cfg(test)]
mod tests;

use crate::ClusterOrchestrate;
use crate::ClusterOrchestrateMut;

/// Progress and execute orchestration actions.
pub fn orchestrate(data: &ClusterOrchestrate, data_mut: &mut ClusterOrchestrateMut) -> Result<()> {
    if let Some(span) = data_mut.span.as_mut() {
        span.log(Log::new().log("stage", "orchestrate"));
    }

    // Keep track of exclusive actions that were started so we avoid starting clashing ones.
    let mut exclusive_by_mode = HashSet::new();

    for info in &data.cluster_view.actions_unfinished_orchestrator {
        match info.state {
            OrchestratorActionState::PendingSchedule => {
                self::pending::start_action(data, data_mut, info.action_id, &mut exclusive_by_mode)?
            }
            OrchestratorActionState::Running => {
                self::running::continue_action(data, data_mut, info.action_id)?
            }
            _ => (),
        };
    }

    Ok(())
}

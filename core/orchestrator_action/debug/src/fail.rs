use anyhow::Result;

use replicante_models_core::actions::orchestrator::OrchestratorAction as OARecord;
use replicante_models_core::actions::orchestrator::OrchestratorActionScheduleMode;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicore_iface_orchestrator_action::registry_entry_factory;
use replicore_iface_orchestrator_action::OrchestratorAction;
use replicore_iface_orchestrator_action::ProgressChanges;

/// A simple action that fails the first time it is progressed.
#[derive(Default)]
pub struct Fail {}

registry_entry_factory! {
    handler: Fail,
    schedule_mode: OrchestratorActionScheduleMode::Exclusive,
    summary: "Fail at the first progression",
}

impl OrchestratorAction for Fail {
    fn progress(&self, _: &OARecord) -> Result<Option<ProgressChanges>> {
        let changes = ProgressChanges {
            state: OrchestratorActionState::Failed,
            state_payload: None,
        };
        Ok(Some(changes))
    }
}

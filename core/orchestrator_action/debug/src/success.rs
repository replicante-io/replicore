use anyhow::Result;

use replicante_models_core::actions::orchestrator::OrchestratorAction as OARecord;
use replicante_models_core::actions::orchestrator::OrchestratorActionScheduleMode;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicore_iface_orchestrator_action::registry_entry_factory;
use replicore_iface_orchestrator_action::OrchestratorAction;
use replicore_iface_orchestrator_action::ProgressChanges;

/// A simple action that succeeds the first time it is progressed.
#[derive(Default)]
pub struct Success {}

registry_entry_factory! {
    handler: Success,
    schedule_mode: OrchestratorActionScheduleMode::Exclusive,
    summary: "Complete at first progression",
    timeout: crate::ONE_HOUR,
}

impl OrchestratorAction for Success {
    fn progress(&self, _: &OARecord) -> Result<Option<ProgressChanges>> {
        let changes = ProgressChanges {
            state: OrchestratorActionState::Done,
            state_payload: None,
            state_payload_error: None,
        };
        Ok(Some(changes))
    }
}

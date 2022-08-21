use anyhow::Result;

use replicante_models_core::actions::orchestrator::OrchestratorAction as OARecord;
use replicante_models_core::actions::orchestrator::OrchestratorActionScheduleMode;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicore_iface_orchestrator_action::registry_entry_factory;
use replicore_iface_orchestrator_action::OrchestratorAction;
use replicore_iface_orchestrator_action::ProgressChanges;

/// A simple action that return the arguments it was called for as output.
#[derive(Default)]
pub struct Ping {}

registry_entry_factory! {
    handler: Ping,
    schedule_mode: OrchestratorActionScheduleMode::Exclusive,
    summary: "Return as output the given arguments and complete immediately",
    timeout: crate::ONE_HOUR,
}

impl OrchestratorAction for Ping {
    fn progress(&self, record: &OARecord) -> Result<Option<ProgressChanges>> {
        let pong = serde_json::json!({
            "pong": record.args.clone(),
        });
        let changes = ProgressChanges {
            state: OrchestratorActionState::Done,
            state_payload: Some(pong),
        };
        Ok(Some(changes))
    }
}

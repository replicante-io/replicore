use replicante_models_core::actions::orchestrator::OrchestratorActionScheduleMode;
use replicore_iface_orchestrator_action::registry_entry_factory;
use replicore_iface_orchestrator_action::OrchestratorAction;

/// A simple action that fails the first time it is progressed.
#[derive(Default)]
pub struct Failing {}

registry_entry_factory! {
    handler: Failing,
    schedule_mode: OrchestratorActionScheduleMode::Exclusive,
    summary: "Fail at the first progression",
}

impl OrchestratorAction for Failing {}

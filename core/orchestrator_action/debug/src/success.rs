use replicante_models_core::actions::orchestrator::OrchestratorActionScheduleMode;
use replicore_iface_orchestrator_action::registry_entry_factory;
use replicore_iface_orchestrator_action::OrchestratorAction;

/// A simple action that succeeds the first time it is progressed.
#[derive(Default)]
pub struct Success {}

registry_entry_factory! {
    handler: Success,
    schedule_mode: OrchestratorActionScheduleMode::Exclusive,
    summary: "Complete at first progression",
}

impl OrchestratorAction for Success {}

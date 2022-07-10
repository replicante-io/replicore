use replicante_models_core::actions::orchestrator::OrchestratorActionScheduleMode;
use replicore_iface_orchestrator_action::registry_entry_factory;
use replicore_iface_orchestrator_action::OrchestratorAction;

/// A simple action that return the arguments it was called for as output.
#[derive(Default)]
pub struct Ping {}

registry_entry_factory! {
    handler: Ping,
    schedule_mode: OrchestratorActionScheduleMode::Exclusive,
    summary: "Return as output the given arguments and complete immediately",
}

impl OrchestratorAction for Ping {}

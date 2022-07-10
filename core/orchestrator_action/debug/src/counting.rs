use replicante_models_core::actions::orchestrator::OrchestratorActionScheduleMode;
use replicore_iface_orchestrator_action::registry_entry_factory;
use replicore_iface_orchestrator_action::OrchestratorAction;

/// A simple action that progressing incrementally a number of time before success.
#[derive(Default)]
pub struct Counting {}

registry_entry_factory! {
    handler: Counting,
    schedule_mode: OrchestratorActionScheduleMode::Exclusive,
    summary: "Increment a counter every progression before completing",
}

impl OrchestratorAction for Counting {}

use replicore_iface_orchestrator_action::OrchestratorAction;

/// A simple action that fails the first time it is progressed.
#[derive(Default)]
pub struct Failing {}

impl OrchestratorAction for Failing {}

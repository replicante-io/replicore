use replicore_iface_orchestrator_action::OrchestratorAction;

/// A simple action that succeeds the first time it is progressed.
#[derive(Default)]
pub struct Success {}

impl OrchestratorAction for Success {}

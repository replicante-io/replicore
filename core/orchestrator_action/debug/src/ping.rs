use replicore_iface_orchestrator_action::OrchestratorAction;

/// A simple action that return the arguments it was called for as output.
#[derive(Default)]
pub struct Ping {}

impl OrchestratorAction for Ping {}

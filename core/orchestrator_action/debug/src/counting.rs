use replicore_iface_orchestrator_action::OrchestratorAction;

/// A simple action that progressing incrementally a number of time before success.
#[derive(Default)]
pub struct Counting {}

impl OrchestratorAction for Counting {}

use replicore_iface_orchestrator_action::OrchestratorAction;
use replicore_iface_orchestrator_action::OrchestratorActionDescriptor;

/// A simple action that progressing incrementally a number of time before success.
#[derive(Default)]
pub struct Counting {}

impl OrchestratorAction for Counting {
    fn describe(&self) -> OrchestratorActionDescriptor {
        OrchestratorActionDescriptor {
            summary: "Increment a counter every progression before completing".into(),
        }
    }
}

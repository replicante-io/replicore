use replicore_iface_orchestrator_action::OrchestratorAction;
use replicore_iface_orchestrator_action::OrchestratorActionDescriptor;

/// A simple action that return the arguments it was called for as output.
#[derive(Default)]
pub struct Ping {}

impl OrchestratorAction for Ping {
    fn describe(&self) -> OrchestratorActionDescriptor {
        OrchestratorActionDescriptor {
            summary: "Return as output the given arguments and complete".into(),
        }
    }
}

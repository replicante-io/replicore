use replicore_iface_orchestrator_action::OrchestratorAction;
use replicore_iface_orchestrator_action::OrchestratorActionDescriptor;

/// A simple action that fails the first time it is progressed.
#[derive(Default)]
pub struct Failing {}

impl OrchestratorAction for Failing {
    fn describe(&self) -> OrchestratorActionDescriptor {
        OrchestratorActionDescriptor {
            summary: "Fail at the first progression".into(),
        }
    }
}

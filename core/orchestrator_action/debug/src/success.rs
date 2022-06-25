use replicore_iface_orchestrator_action::OrchestratorAction;
use replicore_iface_orchestrator_action::OrchestratorActionDescriptor;

/// A simple action that succeeds the first time it is progressed.
#[derive(Default)]
pub struct Success {}

impl OrchestratorAction for Success {
    fn describe(&self) -> OrchestratorActionDescriptor {
        OrchestratorActionDescriptor {
            summary: "Complete at first progression".into(),
        }
    }
}

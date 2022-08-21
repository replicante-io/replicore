use anyhow::Result;

use replicante_models_core::actions::orchestrator::OrchestratorAction as OARecord;
use replicante_models_core::actions::orchestrator::OrchestratorActionScheduleMode;
use replicore_iface_orchestrator_action::registry_entry_factory;
use replicore_iface_orchestrator_action::OrchestratorAction;
use replicore_iface_orchestrator_action::ProgressChanges;

/// A simple action that fails the first time it is progressed.
#[derive(Default)]
pub struct Fail {}

registry_entry_factory! {
    handler: Fail,
    schedule_mode: OrchestratorActionScheduleMode::Exclusive,
    summary: "Fail at the first progression",
    timeout: crate::ONE_HOUR,
}

impl OrchestratorAction for Fail {
    fn progress(&self, record: &OARecord) -> Result<Option<ProgressChanges>> {
        let error = anyhow::anyhow!("debug action failed intentionally");
        if record
            .args
            .get("wrapped")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
        {
            anyhow::bail!(error.context("debug action wrapped in some context"));
        }
        anyhow::bail!(error);
    }
}

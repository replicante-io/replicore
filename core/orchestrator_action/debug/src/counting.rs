use anyhow::Result;

use replicante_models_core::actions::orchestrator::OrchestratorAction as OARecord;
use replicante_models_core::actions::orchestrator::OrchestratorActionScheduleMode;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicore_iface_orchestrator_action::errors::InvalidArgumentsType;
use replicore_iface_orchestrator_action::errors::InvalidOrMissingArgument;
use replicore_iface_orchestrator_action::registry_entry_factory;
use replicore_iface_orchestrator_action::OrchestratorAction;
use replicore_iface_orchestrator_action::ProgressChanges;

/// A simple action that progressing incrementally a number of time before success.
#[derive(Default)]
pub struct Counting {}

registry_entry_factory! {
    handler: Counting,
    schedule_mode: OrchestratorActionScheduleMode::Exclusive,
    summary: "Increment a counter every progression before completing",
    timeout: crate::ONE_DAY,
}

impl OrchestratorAction for Counting {
    fn progress(&self, record: &OARecord) -> Result<Option<ProgressChanges>> {
        // Check the provided args are valid.
        let args = &record.args;
        if !args.is_null() && !args.is_object() {
            anyhow::bail!(InvalidArgumentsType {
                expected_types: "null, object".into(),
            });
        }
        let count_total = match args.get("count").and_then(|count| count.as_i64()) {
            Some(count) => count,
            None => {
                anyhow::bail!(InvalidOrMissingArgument {
                    expected_types: "int".into(),
                    path: "count".into(),
                    required: true,
                });
            }
        };

        // Get the current count value or start counting.
        // Assume missing and invalid current values indicate we are starting.
        let current_value = match &record.state_payload {
            None => 0,
            Some(payload) => payload
                .get("count_index")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0),
        };
        let next_value = current_value + 1;

        // Increment count and possibly finish the action.
        let state = if next_value < count_total {
            OrchestratorActionState::Running
        } else {
            OrchestratorActionState::Done
        };
        let state_payload = Some(serde_json::json!({
            "count_index": next_value,
        }));
        let changes = ProgressChanges {
            state,
            state_payload,
        };
        Ok(Some(changes))
    }
}

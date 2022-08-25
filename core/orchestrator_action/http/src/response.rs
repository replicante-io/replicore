use reqwest::StatusCode;

use replicante_models_core::actions::orchestrator::OrchestratorAction;
use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicore_iface_orchestrator_action::ProgressChanges;

/// Response from the remote for further processing.
pub struct ResponseInfo {
    /// HTTP response status code.
    pub status: StatusCode,

    /// HTTP response body as text.
    pub text: String,
}

/// Decode the response from the remote into optional progress changes.
pub fn decode(response: ResponseInfo) -> Option<ProgressChanges> {
    match response.status {
        StatusCode::NO_CONTENT => None,
        StatusCode::OK => {
            if let Ok(changes) = serde_json::from_str(&response.text) {
                Some(changes)
            } else if let Some(changes) = fail_as_json(&response) {
                Some(changes)
            } else {
                let changes = fail_as_text(&response);
                Some(changes)
            }
        }
        _ => {
            if let Some(changes) = fail_as_json(&response) {
                Some(changes)
            } else {
                let changes = fail_as_text(&response);
                Some(changes)
            }
        }
    }
}

/// Ensure PendingSchedule actions are moved to a new state or move them to running.
pub fn ensure_move_to_running(
    record: &OrchestratorAction,
    changes: Option<ProgressChanges>,
) -> Option<ProgressChanges> {
    // We only care if the current state is PendingSchedule.
    if record.state != OrchestratorActionState::PendingSchedule {
        return changes;
    }

    // Figure out what the action state would be given the changes.
    let next_state = changes
        .as_ref()
        .map(|changes| changes.state)
        .unwrap_or(record.state);

    // If the next state is not still PendingSchedule return the changes as-is.
    if next_state != OrchestratorActionState::PendingSchedule {
        return changes;
    }

    changes
        .map(|mut changes| {
            changes.state = OrchestratorActionState::Running;
            changes
        })
        .or({
            Some(ProgressChanges {
                state: OrchestratorActionState::Running,
                state_payload: None,
                state_payload_error: None,
            })
        })
}

/// Attempt to return failing changes from a JSON payload.
fn fail_as_json(response: &ResponseInfo) -> Option<ProgressChanges> {
    let payload = match serde_json::from_str::<serde_json::Value>(&response.text) {
        Err(_) => return None,
        Ok(payload) => payload,
    };
    let payload = serde_json::json!({
        "payload": payload,
        "response_status": response.status.as_u16(),
    });
    let changes = ProgressChanges {
        state: OrchestratorActionState::Failed,
        state_payload: None,
        state_payload_error: Some(Some(payload)),
    };
    Some(changes)
}

/// Return failing changes with from a text payload.
fn fail_as_text(response: &ResponseInfo) -> ProgressChanges {
    ProgressChanges {
        state: OrchestratorActionState::Failed,
        state_payload: None,
        state_payload_error: Some(Some(serde_json::json!({
            "response_body": &response.text,
            "response_status": response.status.as_u16(),
        }))),
    }
}

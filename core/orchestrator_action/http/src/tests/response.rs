use reqwest::StatusCode;

use replicante_models_core::actions::orchestrator::OrchestratorActionState;
use replicore_iface_orchestrator_action::ProgressChanges;

use super::action_record;
use crate::response::ResponseInfo;

fn test_ensure_move_to_running_none_with_state(state: OrchestratorActionState) {
    let mut record = action_record();
    record.state = state;
    let changes = crate::response::ensure_move_to_running(&record, None);
    assert_eq!(changes, None);
}

fn test_not_changes_with_code(status: StatusCode) {
    let info = ResponseInfo {
        status,
        text: serde_json::to_string(&serde_json::json!({
            "raw": "data",
            "is_valid": false,
        }))
        .unwrap(),
    };
    let changes = crate::response::decode(info).expect("response to have changes");
    assert_eq!(changes.state, OrchestratorActionState::Failed);
    assert_eq!(changes.state_payload, None);
    assert_eq!(
        changes.state_payload_error,
        Some(Some(serde_json::json!({
            "payload": {
                "raw": "data",
                "is_valid": false,
            },
            "response_status": status.as_u16(),
        }))),
    );
}

fn test_not_json_with_code(status: StatusCode) {
    let info = ResponseInfo {
        status,
        text: "raw data".into(),
    };
    let changes = crate::response::decode(info).expect("response to have changes");
    assert_eq!(changes.state, OrchestratorActionState::Failed);
    assert_eq!(changes.state_payload, None);
    assert_eq!(
        changes.state_payload_error,
        Some(Some(serde_json::json!({
            "response_body": "raw data",
            "response_status": status.as_u16(),
        }))),
    );
}

#[test]
fn decode_response_200() {
    let info = ResponseInfo {
        status: StatusCode::OK,
        text: serde_json::to_string(&ProgressChanges {
            state: OrchestratorActionState::Done,
            state_payload: Some(Some(serde_json::json!({
                "some": "payload",
                "set": true,
            }))),
            state_payload_error: None,
        })
        .unwrap(),
    };
    let changes = crate::response::decode(info).expect("response to have changes");
    assert_eq!(changes.state, OrchestratorActionState::Done);
    assert_eq!(
        changes.state_payload,
        Some(Some(serde_json::json!({
            "some": "payload",
            "set": true,
        }))),
    );
}

#[test]
fn decode_response_200_not_changes() {
    test_not_changes_with_code(StatusCode::OK);
}

#[test]
fn decode_response_200_not_json() {
    test_not_json_with_code(StatusCode::OK);
}

#[test]
fn decode_response_204() {
    let info = ResponseInfo {
        status: StatusCode::NO_CONTENT,
        text: "".into(),
    };
    let changes = crate::response::decode(info);
    assert_eq!(changes, None);
}

#[test]
fn decode_response_not_ok() {
    test_not_changes_with_code(StatusCode::FORBIDDEN);
    test_not_json_with_code(StatusCode::NOT_FOUND);
    test_not_json_with_code(StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn ensure_move_to_running_action_not_pending() {
    test_ensure_move_to_running_none_with_state(OrchestratorActionState::Cancelled);
    test_ensure_move_to_running_none_with_state(OrchestratorActionState::Done);
    test_ensure_move_to_running_none_with_state(OrchestratorActionState::Failed);
    test_ensure_move_to_running_none_with_state(OrchestratorActionState::Running);
}

#[test]
fn ensure_move_to_running_keeps_given_non_pending_schedule() {
    let mut record = action_record();
    record.state = OrchestratorActionState::PendingSchedule;
    let changes = ProgressChanges {
        state: OrchestratorActionState::Done,
        state_payload: None,
        state_payload_error: None,
    };
    let changes = crate::response::ensure_move_to_running(&record, Some(changes))
        .expect("changes should be provided");
    assert_eq!(changes.state, OrchestratorActionState::Done);
}

#[test]
fn ensure_move_to_running_keeps_given_pending_schedule() {
    let mut record = action_record();
    record.state = OrchestratorActionState::PendingSchedule;
    let changes = ProgressChanges {
        state: OrchestratorActionState::PendingSchedule,
        state_payload: None,
        state_payload_error: None,
    };
    let changes = crate::response::ensure_move_to_running(&record, Some(changes))
        .expect("changes should be provided");
    assert_eq!(changes.state, OrchestratorActionState::Running);
}

#[test]
fn ensure_move_to_running_keeps_given_no_changes() {
    let mut record = action_record();
    record.state = OrchestratorActionState::PendingSchedule;
    let changes =
        crate::response::ensure_move_to_running(&record, None).expect("changes should be provided");
    assert_eq!(changes.state, OrchestratorActionState::Running);
}
